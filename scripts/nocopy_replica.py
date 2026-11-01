#!/usr/bin/env python3
"""Independent Python replica of the no-copy sensor-principles/sensor-engine
mechanics, used to (a) validate against recorded affinity results and
(b) predict candidate orders before any Rust change. Scratch only."""
import math, sys

M64 = (1 << 64) - 1

def splitmix(value):
    value = (value + 0x9E3779B97F4A7C15) & M64
    value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & M64
    value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & M64
    return value ^ (value >> 31)

def rotl64(value, n):
    value &= M64
    return ((value << n) | (value >> (64 - n))) & M64

def unit(bits):
    return (((bits & M64) >> 11) + 0.5) / float(1 << 53)

def synthetic_key(cls, index):
    if cls == 0:
        return bytes(1 + (splitmix((index * 32 + p) & M64) % 255) for p in range(32))
    if cls == 1:
        src = ("synthetic-handshake-frame-%02d" % index).encode()
        return bytes(src[p % len(src)] for p in range(32))
    return bytes(1 + (splitmix((p * 17 + index * 13) & M64) % 255) for p in range(32))

def fingerprint(bs):
    h = 0xCBF29CE484222325
    for b in bs:
        h = ((h ^ b) * 0x100000001B3) & M64
    return h

DONOR, RECIPIENT = 0, 1
import os
RESONANCE = os.environ.get('RESONANCE') == '1'

class Field:
    def __init__(self, seed, thermal=0.0):
        self.thermal = thermal
        self.amplitude = [0.8, 0.6]
        self.born_weight = 0.64
        self.memory = [None] * 4
        self.memory_cursor = 0
        self.information_drive = 0.0
        self.last_similarity = 0.0
        self.last_pulse = 0.0
        self.cooperative_factor = 1.0
        self.donor_inventory = 0.0
        self.recipient_inventory = 0.0
        self.position = 0.0
        self.momentum = 0.0
        self.potential = [0.0] * 4
        self.diffusion_error = 0.0
        self.reactants = [0.0, 0.0]
        self.catalyst = 0.7
        self.chemical_flux = 0.0
        self.activation = [0.0] * 4
        self.logistic_state = 0.05 + 0.9 * unit(seed ^ 0x9E3779B97F4A7C15)
        self.logistic_rate = 4.0
        self.tent_state = 0.05 + 0.9 * unit(rotl64(seed, 23))
        self.drive = 0.0

    def quantum_base(self, obs):
        phase = ((obs['fingerprint'] & 0xFFFF) / 65535.0) * math.pi / 4.0
        angle = 0.13 + phase + (obs['tick'] % 7) * 0.01
        s, c = math.sin(angle), math.cos(angle)
        a, b = self.amplitude
        self.amplitude = [c * a - s * b, s * a + c * b]
        self.born_weight = self.amplitude[0] ** 2
        if self.thermal > 0.0:
            p0 = 1.0 / (1.0 + math.exp(-1.0 / self.thermal))
            eps = float(os.environ.get('EPS', '0.1'))
            mixed = (1.0 - eps) * self.born_weight + eps * p0
            self.amplitude[0] = math.sqrt(mixed)
            self.amplitude[1] = math.sqrt(1.0 - mixed)
            self.born_weight = mixed

    def neutral_base(self, obs):
        self.born_weight = 0.5

    def information(self, obs):
        decay = float(os.environ.get('MEM_DECAY', '0.9'))
        for m in self.memory:
            if m is not None:
                m[1] *= decay
        if obs['direction'] == DONOR:
            self.last_similarity = 0.0
            self.last_pulse = 0.0
            self.memory[self.memory_cursor] = [obs['fingerprint'], obs['quantity']]
            self.memory_cursor = (self.memory_cursor + 1) % 4
            self.drive += 0.02 * obs['quantity']
        else:
            sim = 0.0
            for m in self.memory:
                if m is not None:
                    dist = bin(m[0] ^ obs['fingerprint']).count('1')
                    sim = max(sim, (1.0 - dist / 64.0) * m[1])
            pulse = sim * obs['quantity']
            self.last_similarity = sim
            self.information_drive += pulse
            weight = sim ** float(os.environ.get('INFO_POW', '1'))
            if os.environ.get('INFO_GATE') == '1':
                K = float(os.environ.get('INFO_K', '2'))
                weight *= max(K * sim - (K - 1.0), 0.0)
            self.drive += pulse * (0.25 + self.born_weight) * weight

    def conservation(self, obs):
        if obs['direction'] == DONOR:
            self.donor_inventory += obs['quantity']
        else:
            self.recipient_inventory += obs['quantity']
        gap = self.recipient_inventory - self.donor_inventory
        self.drive += 0.25 * gap * gap

    def affinity_conservation(self, obs):
        if obs['direction'] == DONOR:
            self.donor_inventory += obs['quantity']
        else:
            self.recipient_inventory += obs['quantity']
        gap = self.recipient_inventory - self.donor_inventory
        K = float(os.environ.get('AFF_K', '2'))
        excess = max(K * self.last_similarity - (K - 1.0), 0.0)
        C = float(os.environ.get('AFF_COEF', '0.25'))
        self.drive += C * gap * gap * excess

    def sharp_conservation(self, obs):
        if obs['direction'] == DONOR:
            self.donor_inventory += obs['quantity']
        else:
            self.recipient_inventory += obs['quantity']
        gap = self.recipient_inventory - self.donor_inventory
        K = float(os.environ.get('SHARP_K', '4'))
        excess = max(K * self.last_similarity - (K - 1.0), 0.0)
        self.drive += 0.25 * gap * gap * excess

    def cooperative_binding(self):
        s = self.last_similarity
        self.cooperative_factor = 1.0 + s * s / (0.25 + s * s)

    def mechanics(self):
        self.momentum += -0.125 * self.position + 0.25 * self.drive
        self.position += 0.25 * self.momentum
        self.drive += 0.05 * abs(self.momentum)

    def physics(self, obs):
        site = obs['site'] % 4
        self.potential[site] += 0.1 * obs['quantity'] * (0.1 + abs(self.drive))
        before = sum(self.potential)
        old = self.potential[:]
        for i in range(4):
            left = old[(i + 3) % 4]
            right = old[(i + 1) % 4]
            self.potential[i] = old[i] + 0.1 * (left + right - 2.0 * old[i])
        after = sum(self.potential)
        self.diffusion_error = max(self.diffusion_error, abs(after - before))
        self.drive += 0.1 * (max(self.potential) - min(self.potential))

    def chemistry(self, obs):
        idx = 0 if obs['direction'] == DONOR else 1
        self.reactants[idx] = min(self.reactants[idx] + 0.4 * obs['quantity'], 1.0)
        extent = (0.3 + 0.2 * min(abs(self.drive), 1.0)) * self.catalyst \
            * self.cooperative_factor * self.reactants[0] * self.reactants[1]
        extent = min(extent, self.reactants[0], self.reactants[1])
        self.reactants[0] -= extent
        self.reactants[1] -= extent
        self.chemical_flux += extent
        self.drive += 0.2 * extent

    def cognition(self, obs):
        site = obs['site'] % 4
        self.activation[site] = min(self.activation[site] + 0.1 * obs['quantity'] * abs(self.drive), 1.0)
        old = self.activation[:]
        for i in range(4):
            competing = sum(old) - old[i]
            self.activation[i] = max(old[i] - 0.08 * competing, 0.0)
        self.drive += 0.1 * (max(self.activation) - min(self.activation))

    def logistic(self):
        self.logistic_state = self.logistic_rate * self.logistic_state * (1.0 - self.logistic_state)
        self.drive *= 0.75 + 0.5 * self.logistic_state

    def tent(self):
        self.tent_state = 2.0 * self.tent_state if self.tent_state <= 0.5 else 2.0 * (1.0 - self.tent_state)
        self.drive *= 0.75 + 0.5 * self.tent_state

    def measurements(self):
        field_energy = sum(v * v for v in self.activation)
        reaction = self.chemical_flux * (1.0 + abs(self.drive)) + 0.01 * field_energy
        return reaction

# stage ids
INFORMATION, CONSERVATION, AFFINITY, COOPERATIVE, MECHANICS, PHYSICS, CHEMISTRY, COGNITION, LOGISTIC, TENT, SHARP = range(11)

RES = 10
ORDERS = {
    'E': [INFORMATION, RES, AFFINITY, MECHANICS, PHYSICS, CHEMISTRY, COGNITION, LOGISTIC, TENT],
    'F': [INFORMATION, RES, CHEMISTRY, TENT, AFFINITY, PHYSICS, MECHANICS, LOGISTIC, COGNITION],
    'G': [INFORMATION, COOPERATIVE, AFFINITY, MECHANICS, PHYSICS, CHEMISTRY, COGNITION, LOGISTIC, TENT],
    'H': [INFORMATION, COOPERATIVE, CHEMISTRY, TENT, AFFINITY, PHYSICS, MECHANICS, LOGISTIC, COGNITION],
    'K': [INFORMATION, COOPERATIVE, AFFINITY, CHEMISTRY, MECHANICS, PHYSICS, COGNITION, LOGISTIC, TENT],
    'N': [INFORMATION, RES, SHARP, MECHANICS, PHYSICS, CHEMISTRY, COGNITION, LOGISTIC, TENT],
    'O': [INFORMATION, RES, CHEMISTRY, TENT, SHARP, PHYSICS, MECHANICS, LOGISTIC, COGNITION],
}

ARMS = ['Full', 'NoBase', 'BaseOnly', 'NoChaos', 'LogisticOnly', 'TentOnly',
        'NoInformation', 'NoMechanics', 'NoPhysics', 'NoChemistry', 'NoCognition',
        'NoConservation', 'NoCooperativeBinding']

def arm_includes(arm, stage):
    if arm in ('Full', 'NoBase'):
        return True
    if arm == 'BaseOnly':
        return False
    if arm == 'NoChaos':
        return stage not in (LOGISTIC, TENT)
    if arm == 'LogisticOnly':
        return stage != TENT
    if arm == 'TentOnly':
        return stage != LOGISTIC
    if arm == 'NoInformation':
        return stage != INFORMATION
    if arm == 'NoMechanics':
        return stage != MECHANICS
    if arm == 'NoPhysics':
        return stage != PHYSICS
    if arm == 'NoChemistry':
        return stage != CHEMISTRY
    if arm == 'NoCognition':
        return stage != COGNITION
    if arm == 'NoConservation':
        return stage not in (CONSERVATION, AFFINITY, SHARP)
    if arm == 'NoCooperativeBinding':
        return stage != COOPERATIVE
    raise ValueError(arm)

STAGE_FN = {INFORMATION: 'information', CONSERVATION: 'conservation',
            AFFINITY: 'affinity_conservation', SHARP: 'sharp_conservation', COOPERATIVE: 'cooperative_binding',
            MECHANICS: 'mechanics', PHYSICS: 'physics', CHEMISTRY: 'chemistry',
            COGNITION: 'cognition', LOGISTIC: 'logistic', TENT: 'tent'}

class KeyCell:
    def __init__(self, nbytes=32):
        self.heat = 0.0
        self.ruptured = False
        self.nonzero = nbytes
    def absorb(self, reaction):
        if not math.isfinite(reaction) or reaction < 0.0:
            self.ruptured = True; self.nonzero = 0; return
        self.heat = 0.25 * self.heat + reaction
        if not math.isfinite(self.heat):
            self.ruptured = True; self.nonzero = 0; return
        if self.heat >= 0.25:
            self.ruptured = True; self.nonzero = 0

def trace(cls, index, placement, scenario):
    if scenario == 5:
        return []
    identity = fingerprint(synthetic_key(cls, index))
    donor = {'tick': 1, 'direction': DONOR, 'site': placement, 'fingerprint': identity, 'quantity': 1.0}
    first = {'tick': 2, 'direction': RECIPIENT, 'site': (placement + 1) % 4, 'fingerprint': identity,
             'quantity': 0.25 if scenario == 3 else 1.0}
    second = {'tick': 3, 'direction': RECIPIENT, 'site': (placement + 2) % 4,
              'fingerprint': fingerprint(synthetic_key(cls, (index + 7) % 16)) if scenario == 4 else identity,
              'quantity': 0.75 if scenario == 3 else 1.0}
    if scenario == 0:
        return [donor, first]
    if 1 <= scenario <= 4:
        return [donor, first, second]
    return []

def run(observations, seed, order, arm, destruction_present, thermal=0.0):
    field = Field(seed, thermal)
    stages = ORDERS[order]
    key = KeyCell()
    reaction = field.measurements()
    for obs in observations:
        if arm == 'NoBase':
            field.neutral_base(obs)
        else:
            field.quantum_base(obs)
        for st in stages:
            if st == 10:
                if RESONANCE:
                    sim = field.last_similarity
                    K = float(os.environ.get('GATE_K', '2'))
                    excess = max(K * sim - (K - 1.0), 0.0)
                    C = float(os.environ.get('RES_COEF', '0.25'))
                    field.drive += C * excess * field.last_pulse
                    field.last_pulse = sim * obs['quantity'] if obs['direction'] == RECIPIENT else 0.0
                continue
            if arm_includes(arm, st):
                fn = getattr(field, STAGE_FN[st])
                fn(obs) if st in (INFORMATION, CONSERVATION, AFFINITY, SHARP, PHYSICS, CHEMISTRY, COGNITION) else fn()
        reaction = field.measurements()
        if destruction_present:
            key.absorb(reaction)
    return key, reaction

def experiment(order_names, arms=None):
    if arms is None:
        arms = os.environ.get('ARMS', ','.join(ARMS)).split(',')
    T = float(os.environ.get('T', '0'))
    results = {}
    for order in order_names:
        counts = {a: {s: 0 for s in range(6)} for a in arms}
        heats = {a: {s: [] for s in range(6)} for a in arms}
        for arm in arms:
            for cls in range(3):
                for index in range(16):
                    seed = splitmix((cls * 16 + index) & M64)
                    for placement in range(3):
                        for scenario in range(6):
                            key, _ = run(trace(cls, index, placement, scenario), seed, order, arm, True, T)
                            if key.ruptured:
                                counts[arm][scenario] += 1
                            heats[arm][scenario].append(key.heat)
        # no-destruction arm
        nd = {s: 0 for s in range(6)}
        for cls in range(3):
            for index in range(16):
                seed = splitmix((cls * 16 + index) & M64)
                for placement in range(3):
                    for scenario in range(6):
                        key, _ = run(trace(cls, index, placement, scenario), seed, order, 'Full', False, T)
                        if key.ruptured:
                            nd[scenario] += 1
        # repeated-use sequences (scenario 0, persistent cell, 16 cycles)
        survivals = 0
        for cls in range(3):
            for index in range(16):
                seed = splitmix((cls * 16 + index) & M64)
                for placement in range(3):
                    key = KeyCell()
                    for _ in range(16):
                        run(trace(cls, index, placement, 0), seed, order, 'Full', True)  # fresh field each time
                        # persistent cell: absorb manually
                    # redo properly below
        # proper repeated-use: one field per cycle? Rust: run_destruction_experiment creates new engine per call but key persists
        survivals = 0
        for cls in range(3):
            for index in range(16):
                seed = splitmix((cls * 16 + index) & M64)
                for placement in range(3):
                    key = KeyCell()
                    for _ in range(16):
                        field = Field(seed, T)
                        stages = ORDERS[order]
                        for obs in trace(cls, index, placement, 0):
                            field.quantum_base(obs)
                            for st in stages:
                                if arm_includes('Full', st):
                                    fn = getattr(field, STAGE_FN[st])
                                    fn(obs) if st in (INFORMATION, CONSERVATION, AFFINITY, SHARP, PHYSICS, CHEMISTRY, COGNITION) else fn()
                            key.absorb(field.measurements())
                    if not key.ruptured:
                        survivals += 1
        results[order] = {'counts': counts, 'nd': nd, 'survivals': survivals,
                          'mean_heat': {a: {s: (sum(v)/len(v) if v else 0.0) for s, v in h.items()} for a, h in heats.items()}}
    return results

if __name__ == '__main__':
    which = sys.argv[1] if len(sys.argv) > 1 else 'validate'
    if which == 'validate':
        res = experiment(['E', 'F'])
        for order, r in res.items():
            c = r['counts']['Full']
            print(f"order {order}: dup={c[1]}/144 move={c[0]}/144 split={c[3]}/144 unrelated={c[4]}/144 "
                  f"survivals={r['survivals']}/144 sep={c[1]-c[4]}")
    else:
        res = experiment(['G', 'H', 'K'])
        for order, r in res.items():
            c = r['counts']['Full']
            print(f"order {order}: dup={c[1]}/144 move={c[0]}/144 split={c[3]}/144 unrelated={c[4]}/144 "
                  f"survivals={r['survivals']}/144 sep={c[1]-c[4]}")
            for arm in ['NoBase', 'NoChaos', 'LogisticOnly', 'TentOnly', 'BaseOnly', 'NoCooperativeBinding', 'NoConservation']:
                ca = r['counts'][arm]
                print(f"    arm {arm}: dup={ca[1]} unrelated={ca[4]} sep={ca[1]-ca[4]}")
            print(f"    no-destruction ruptures: {r['nd']}")
