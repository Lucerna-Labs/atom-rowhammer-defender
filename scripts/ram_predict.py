"""Replica prediction for the RAM monitor pipeline (run BEFORE monitor code).

Models: sampled LLC-miss data address -> row fingerprint -> class (fp % 8)
-> SwarmTripwire::observe_frame(class) with the measured S3 live profile
(homes [0,1,2,3], Q=0.30, theta=0.15, K=4, T=0.0, hill on, order L), exactly
mirroring crates/imperium-nocopy-swarm/src/lib.rs observe_frame mapping:
  class in homes -> (site j, key_identity(copy j))
  else           -> (reads % 4, label(0xF1, class))

Scenarios (one sampled miss per read, quantity always 1.0):
  hammer-c   : every sample inside one 16 KiB region whose class is c
  scatter    : classes cycle 0..7 (one region per class)
  mixed      : 90% hammer class 0 + 10% scatter
  benign     : pseudo-random regions (splitmix addresses)
"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import nocopy_replica as R

M64 = (1 << 64) - 1
N = 4
ROW_SHIFT = 6          # cache line
ROW_BITS = 8           # fingerprint = address bits [6..14): 16 KiB regions
HOMES = [0, 1, 2, 3]
FRAME_TAG = 0xF1

def splitmix(x):
    x = (x + 0x9E3779B97F4A7C15) & M64
    x = ((x ^ (x >> 30)) * 0xBF58476D1CE4E5B9) & M64
    x = ((x ^ (x >> 27)) * 0x94D049BB133111EB) & M64
    return x ^ (x >> 31)

def key_identity(bs):
    h = 0xCBF29CE484222325
    for b in bs:
        h = ((h ^ b) * 0x100000001B3) & M64
    for _ in range(2):
        h ^= h >> 33
        h = (h * 0xFF51AFD7ED558CCD) & M64
        h ^= h >> 33
        h = (h * 0xC4CEB9FE1A85EC53) & M64
        h ^= h >> 33
    return h

def label(tag, value):
    h = (0xCBF29CE484222325 ^ (tag << 56) ^ value) & M64
    for _ in range(4):
        h ^= h >> 33
        h = (h * 0xFF51AFD7ED558CCD) & M64
        h ^= h >> 33
        h = (h * 0xC4CEB9FE1A85EC53) & M64
        h ^= h >> 33
    return h

def row_fingerprint(addr):
    return (addr >> ROW_SHIFT) & ((1 << ROW_BITS) - 1)

def class_of(addr):
    return row_fingerprint(addr) % 8

def scenario_addr(scen, k):
    if scen.startswith('hammer'):
        c = int(scen.split('-')[1])
        return (c << ROW_SHIFT) + (k % 64)          # same region every read
    if scen == 'scatter':
        return ((k % 8) << ROW_SHIFT) + (k % 64)     # one region per class
    if scen == 'mixed':
        if splitmix(k ^ 0xA5A5) % 10 == 0:
            return ((k % 8) << ROW_SHIFT) + (k % 64)
        return (k % 64)                           # hammer class 0
    if scen == 'benign':
        return (splitmix(k ^ 0xBEEF) << ROW_SHIFT) & ((1 << 40) - 1)
    raise ValueError(scen)

def set_profile():
    os.environ['RESONANCE'] = '1'
    os.environ['INFO_POW'] = '2.0'
    os.environ['RES_COEF'] = '1.0'

def observe(field, stages, obs):
    field.quantum_base(obs)
    for st in stages:
        if st == R.RES:
            sim = field.last_similarity
            K2 = float(os.environ.get('GATE_K', '2'))
            excess = max(K2 * sim - (K2 - 1.0), 0.0)
            C = float(os.environ.get('RES_COEF', '0.25'))
            field.drive += C * excess * field.last_pulse
            field.last_pulse = sim * obs['quantity'] if obs['direction'] == R.RECIPIENT else 0.0
            continue
        fn = getattr(field, R.STAGE_FN[st])
        fn(obs) if st in (R.INFORMATION, R.CONSERVATION, R.AFFINITY, R.SHARP, R.PHYSICS, R.CHEMISTRY, R.COGNITION) else fn()

WINDOW = 64        # sliding window of sampled addresses
THRESH = 16        # concentration gate: top fingerprint count >= 25% of window

def run_ram(scen, max_reads=256, Q=0.30, K=4, T=0.0, order='E'):  # replica 'E' == engine Order::L
    set_profile()
    ids = [key_identity(R.synthetic_key(0, i)) for i in range(N)]
    seeds = [splitmix(j ^ 0x5EED) for j in range(N)]
    fields = [R.Field(seeds[i], T) for i in range(N)]
    keys = [R.KeyCell() for _ in range(N)]
    stages = R.ORDERS[order]
    for i in range(N):
        obs = {'tick': 0, 'direction': R.DONOR, 'site': i, 'fingerprint': ids[i], 'quantity': 1.0}
        observe(fields[i], stages, obs)
        keys[i].absorb(fields[i].measurements())
    trip = None
    reads = 0
    emitted = 0
    window = []
    for k in range(max_reads):
        addr = scenario_addr(scen, k)
        window.append(row_fingerprint(addr))
        if len(window) > WINDOW:
            window.pop(0)
        counts = {}
        for fp in window:
            counts[fp] = counts.get(fp, 0) + 1
        top_fp, top_n = max(counts.items(), key=lambda kv: kv[1])
        if top_n < THRESH:
            continue  # no concentration: nothing is fed to the swarm
        emitted += 1
        cls = top_fp % 8
        if cls in HOMES:
            site, fp = HOMES.index(cls), ids[HOMES.index(cls)]
        else:
            site, fp = reads % N, label(FRAME_TAG, cls)
        reads += 1
        obs = {'tick': k + 1, 'direction': R.RECIPIENT, 'site': site, 'fingerprint': fp, 'quantity': 1.0}
        observe(fields[site], stages, obs)
        r = fields[site].measurements()
        coupling = Q * sum(keys[j].heat for j in range(N) if j != site)
        keys[site].absorb(r + coupling)
        if trip is None and (keys[site].ruptured or sum(1 for kk in keys if kk.heat >= 0.15) >= K):
            trip = (k + 1, reads)  # (samples consumed, swarm reads)
    return trip, emitted, [round(kk.heat, 4) for kk in keys]

if __name__ == '__main__':
    scenarios = [f'hammer-{c}' for c in range(8)] + ['scatter', 'mixed', 'benign']
    for scen in scenarios:
        trip, emitted, heats = run_ram(scen)
        t = f'samples={trip[0]},swarm_reads={trip[1]}' if trip else '-'
        print(f'{scen:10s} trip={t:24s} emitted={emitted:3d} heats={heats}')
