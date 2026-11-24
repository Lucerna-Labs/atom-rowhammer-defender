//! Engine configuration. Every knob is validated; ordering law O10 (weaken
//! is harsher than strengthen) is asserted here as an arithmetic invariant.

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub capacity: usize,
    pub cue_dim: usize,
    pub value_count: usize,
    pub initial_mass: f64,
    pub initial_support: f64,
    pub reinforcement_mass: f64,
    pub reinforcement_support: f64,
    /// Instant support multiplier on contradiction (O10: 1 - this must
    /// exceed reinforcement_support).
    pub contradiction_support_factor: f64,
    /// Evidence dissolve multiplier applied to prior evidence on revision.
    pub contradiction_evidence_factor: f64,
    /// Salience multiplier applied to surprising observations (PREDICT root
    /// is load-bearing through this boost).
    pub surprise_boost: f64,
    pub evidence_rate: f64,
    pub evidence_cap: f64,
    pub decay_support_rate: f64,
    pub decay_mass_rate: f64,
    pub expire_support: f64,
    pub expire_ttl_support: f64,
    pub match_threshold: f64,
    pub query_threshold: f64,
    pub min_reliability: f64,
    pub attention_temperature: f64,
    pub conservation_budget: f64,
    pub initial_ttl: u64,
    pub ttl_per_hit: u64,
    /// Bond expansion depth during retrieval (0 disables; the echo-graph
    /// walk that links facet-dissimilar memories). Default 2 — chains fire
    /// as units in the corpus the mechanism models.
    pub bond_expansion_depth: usize,
    /// Support bonus an observed memory pays to its bonded upstream
    /// partners (echo consolidation: downstream experience keeps upstream
    /// memory alive without repetition).
    pub echo_support_bonus: f64,
    /// Logit bonus granted to bond-expanded traces during retrieval.
    pub echo_activation_bonus: f64,
    /// Kinetic proofreading: distinct-tick contradiction-free stages a
    /// candidate must survive before promotion (D35).
    pub kpr_min_stages: u64,
    /// Expected information gain (D39): how far below the query gate a
    /// trace still counts as a near miss worth an Explore probe.
    pub eig_margin: f64,
    /// Plans scoring below this floor are withheld — the honest answer
    /// becomes "no available observation reduces this uncertainty."
    pub eig_min_score: f64,
    /// Robust aggregation (W14): the maximum weight SHARE any single
    /// trace may hold in a query mixture after redistribution. 1.0 =
    /// off. Below 1.0, a confident-but-wrong trace cannot outvote a
    /// coalition alone (breakdown bound).
    pub robust_weight_cap: f64,
    /// Robust aggregation (W14): how many of the LOWEST-weighted members
    /// are trimmed from the mixture entirely (noise-floor removal).
    /// 0 = off.
    pub robust_trim: usize,
    /// Occam/dedup selection (W17): retrieval members whose CUES are more
    /// similar than this threshold are correlated near-copies — they
    /// collapse to their strongest member (copies' weights are DROPPED,
    /// not summed: correlated evidence is not independent evidence).
    /// Applied BEFORE the robust cap. 1.01 = off (similarity ≤ 1.0).
    pub dedup_threshold: f64,
    /// Satisficing (W18): when the eligible-only retrieval margin already
    /// meets this bar, bond expansion is SKIPPED (the fast path — good
    /// enough is earned). Below the bar, full deliberation runs. 0.0 =
    /// off (always deliberate).
    pub satisficing_margin: f64,
    /// Habituation (W15): how fast novelty decays on repeated
    /// CONFIRMATION (0.0 = off). Groves & Thompson dual process —
    /// attention stops saturating on constants.
    pub habituation_rate: f64,
    /// Sensitization gain (W15): the attention-logit bonus scaled by a
    /// trace's novelty (0.0 = off — the novelty field exists but no
    /// retrieval effect). Surprise/contradiction restores novelty to 1.0.
    pub novelty_gain: f64,
    /// Pareto preservation (#6): when true, queries also return the
    /// non-dominated front of eligible traces over the objectives
    /// (reliability, support, novelty) — the SET of contenders no other
    /// trace dominates, so genuine trade-offs stay named instead of
    /// being collapsed into one score. Pure metadata: the front never
    /// changes the assertion, and it is excluded from the response hash.
    /// Bounded by the eligible count. false = off (empty front).
    pub pareto_front: bool,
    /// Bet hedging (#10): the attention tail — the lowest-standing fifth
    /// of the COMMITTED memory — decays at (1 − hedge_rate) speed and
    /// keeps its ttl, so current losers survive as insurance. When the
    /// regime flips, the alternative hypothesis is still there to answer
    /// instead of being re-learned from scratch. Slows decay only: a
    /// falsified trace still crosses the hard expiry floors (hedge
    /// delays, never cancels), and candidates never hedge (KPR
    /// dissipation is law). 0.0 = off — decay exactly as before.
    pub hedge_rate: f64,
    /// Stigmergy (#14): pheromone DEPOSITED on each traversed bond by a
    /// MARKING query (`query_marking` — an explicit action; plain
    /// queries never deposit, Stage-9 holds). 0.0 = off.
    pub stigmergy_rate: f64,
    /// Stigmergy (#14): how strongly retrieval routing reads bond
    /// pheromone — the routing bonus `gain × pheromone` rides on top of
    /// the echo bonus. ROUTING ONLY: pheromone never touches evidence,
    /// support, or reliability. 0.0 = off.
    pub stigmergy_gain: f64,
    /// Stigmergy (#14): maximum pheromone per bond (bounded work-trace).
    pub pheromone_cap: f64,
    /// Stigmergy (#14): evaporation multiplier applied to every bond on
    /// each decay tick — trails fade with time, so no route becomes a
    /// permanent rut unless traffic keeps renewing it.
    pub pheromone_decay: f64,
    /// Quorum sensing (#15): expensive deliberation (bond expansion)
    /// runs only when at least this many DISTINCT supporters are
    /// eligible — distinctness clusters eligible traces at cue
    /// similarity 0.90, so correlated copies count ONCE (correlated
    /// worlds cannot fake a quorum). 0 = off — always deliberate.
    pub quorum_min: usize,
    /// Lateral inhibition (#13): after selection, the top member softly
    /// suppresses its near-neighbors — each member's weight is multiplied
    /// by (1 − g·sim²) where sim is its cue similarity to the WINNER
    /// (quadratic, so distinct voices stay intact and only the redundant
    /// chorus around the winner fades). The mixture then re-normalizes,
    /// redistributing share from the chorus to distinct contributors.
    /// 0.0 = off — weights exactly as before.
    pub lateral_inhibition: f64,
    /// Precision-weighted prediction error (#7): how strongly a REVISION
    /// scales with the source's estimated precision — its salience
    /// relative to the standing belief's reliability. At 0 (off) every
    /// contradiction revises at full historical strength. At 1, a weak
    /// ping against a strong belief barely dissolves and casts only a
    /// small dissent vote, while a strong contradiction keeps nearly
    /// full force (precision never locks a belief — the escape hatch),
    /// and repeated consistent weak signals still accumulate (bounded
    /// skepticism). O10 applies at unit source precision: a sub-precision
    /// ping is treated as noise, not a real contradiction.
    pub precision_gain: f64,
    /// Perception (E1): the integer evidence COUNT a label's conjunction
    /// must reach within a window before the window may be named
    /// (assert-or-abstain floor; counts references, never a fused score).
    pub perception_support_floor: usize,
    /// Perception (E1): maximum readings admitted in one window.
    pub perception_window_cap: usize,
    /// Perception catalyst (E1-chem): how much one CONFIRMATION of the
    /// active perception relaxes that label's support floor (0.0 = off —
    /// the E1 matcher unchanged). A catalyst lowers the activation energy
    /// of an ongoing reaction only: labels without an active perception
    /// keep the full floor, competition stays evidence-counted, and the
    /// effective floor never drops below 1. Bounded by the receipts'
    /// catalysis caution (stack 15: advantage bounded ~1.9x).
    pub perception_catalysis: f64,
    pub explore_initial: f64,
    pub explore_floor: f64,
    pub cooling: f64,
    pub seed: u64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            capacity: 18,
            cue_dim: 12,
            value_count: 4,
            initial_mass: 1.0,
            initial_support: 0.7,
            reinforcement_mass: 0.18,
            reinforcement_support: 0.12,
            contradiction_support_factor: 0.82,
            contradiction_evidence_factor: 0.55,
            surprise_boost: 1.5,
            evidence_rate: 0.55,
            evidence_cap: 4.0,
            decay_support_rate: 0.035,
            decay_mass_rate: 0.02,
            expire_support: 0.10,
            expire_ttl_support: 0.55,
            match_threshold: 0.72,
            query_threshold: 0.68,
            min_reliability: 0.16,
            attention_temperature: 2.5,
            conservation_budget: 12.0,
            initial_ttl: 8,
            ttl_per_hit: 2,
            bond_expansion_depth: 2,
            echo_support_bonus: 0.15,
            echo_activation_bonus: 0.15,
            kpr_min_stages: 3,
            eig_margin: 0.08,
            eig_min_score: 0.25,
            robust_weight_cap: 1.0, // off by default — validated surfaces unchanged
            robust_trim: 0,
            dedup_threshold: 1.01,   // off by default (similarity ≤ 1.0)
            satisficing_margin: 0.0, // off by default — always deliberate
            habituation_rate: 0.0,   // off by default
            novelty_gain: 0.0,       // off by default
            pareto_front: false,     // off by default — empty front
            hedge_rate: 0.0,         // off by default — decay as before
            stigmergy_rate: 0.0,     // off by default — no deposits
            stigmergy_gain: 0.0,     // off by default — routing ignores trails
            pheromone_cap: 4.0,      // bounded work-traces when armed
            pheromone_decay: 0.90,   // trails fade unless traffic renews them
            lateral_inhibition: 0.0, // off by default — no suppression
            quorum_min: 0,           // off by default — always deliberate
            precision_gain: 0.0,     // off by default — full-strength revisions
            perception_support_floor: 3, // E1 default — three evidence references
            perception_window_cap: 32,   // E1 default — bounded windows
            perception_catalysis: 0.0,   // E1-chem default — OFF (matcher unchanged)
            explore_initial: 1.35,
            explore_floor: 0.18,
            cooling: 0.955,
            seed: 2026_0821,
        }
    }
}

impl EngineConfig {
    /// Fail-closed validation, including ordering law O10.
    pub fn validate(&self) -> Result<(), String> {
        let err = |what: &str| Err(format!("engine config invalid: {what}"));
        if !(2..=1024).contains(&self.capacity) {
            return err("capacity out of range 2..=1024");
        }
        if !(4..=128).contains(&self.cue_dim) {
            return err("cue_dim out of range 4..=128");
        }
        if !(2..=512).contains(&self.value_count) {
            return err("value_count out of range 2..=512");
        }
        if self.initial_mass <= 0.0 || self.initial_support <= 0.0 {
            return err("initial mass/support must be positive");
        }
        if self.reinforcement_mass < 0.0 || self.reinforcement_support < 0.0 {
            return err("reinforcement rates must be non-negative");
        }
        if !(0.0..=1.0).contains(&self.contradiction_support_factor)
            || !(0.0..=1.0).contains(&self.contradiction_evidence_factor)
        {
            return err("contradiction factors must be within [0, 1]");
        }
        if self.surprise_boost < 1.0 {
            return err("surprise boost must be >= 1.0");
        }
        // O10: one contradiction must cost more than one reinforcement gains.
        if (1.0 - self.contradiction_support_factor) <= self.reinforcement_support {
            return err("ordering law O10 violated: contradiction cost must exceed \
                 reinforcement gain");
        }
        if self.decay_support_rate <= 0.0 || self.decay_mass_rate <= 0.0 {
            return err("decay rates must be positive");
        }
        if !(0.0..1.0).contains(&self.expire_support)
            || !(0.0..1.0).contains(&self.expire_ttl_support)
        {
            return err("expiration thresholds must be within (0, 1)");
        }
        if !(0.5..=0.999).contains(&self.match_threshold)
            || !(0.5..=0.999).contains(&self.query_threshold)
        {
            return err("match/query thresholds must be within [0.5, 0.999]");
        }
        if !(0.0..1.0).contains(&self.min_reliability) {
            return err("minimum reliability must be within [0, 1)");
        }
        if self.attention_temperature <= 0.0 {
            return err("attention temperature must be positive");
        }
        if self.conservation_budget <= 0.0 {
            return err("conservation budget must be positive");
        }
        if self.initial_ttl == 0 || self.ttl_per_hit == 0 {
            return err("ttl knobs must be positive");
        }
        if self.bond_expansion_depth > 8 {
            // Raised from 4 (2026-08-30, operator: "increase the hops").
            // Depth 4 was the validated ceiling when chains stopped at 4;
            // the wall was compound (cap + 1/hop echo decay + routing
            // margin), not a single law. The cap raise is gated by
            // tests/bonds.rs depth-8 chain crossings; defaults unchanged.
            return err("bond expansion depth must be within 0..=8");
        }
        if !(1..=64).contains(&self.perception_support_floor) {
            return err("perception support floor must be within 1..=64");
        }
        if !(1..=4096).contains(&self.perception_window_cap) {
            return err("perception window cap must be within 1..=4096");
        }
        if !(0.0..=4.0).contains(&self.perception_catalysis) {
            return err("perception catalysis must be within [0, 4]");
        }
        if !(0.0..=0.5).contains(&self.echo_support_bonus)
            || !(0.0..=1.0).contains(&self.echo_activation_bonus)
        {
            return err("echo bonuses out of range");
        }
        if !(1..=16).contains(&self.kpr_min_stages) {
            return err("kpr_min_stages out of range 1..=16");
        }
        if !(0.01..=0.5).contains(&self.eig_margin) {
            return err("eig_margin out of range 0.01..=0.5");
        }
        if !(0.0..=1.0).contains(&self.eig_min_score) {
            return err("eig_min_score out of range 0..=1");
        }
        if !(0.15..=1.0).contains(&self.robust_weight_cap) {
            return err("robust_weight_cap out of range 0.15..=1.0");
        }
        if self.robust_trim > 8 {
            return err("robust_trim out of range 0..=8");
        }
        if !(0.5..=1.01).contains(&self.dedup_threshold) {
            return err("dedup_threshold out of range 0.5..=1.01");
        }
        if !(0.0..=1.0).contains(&self.satisficing_margin) {
            return err("satisficing_margin out of range 0..=1");
        }
        if !(0.0..=0.5).contains(&self.habituation_rate) {
            return err("habituation_rate out of range 0..=0.5");
        }
        if !(0.0..=1.0).contains(&self.novelty_gain) {
            return err("novelty_gain out of range 0..=1");
        }
        if !(0.0..=0.5).contains(&self.hedge_rate) {
            return err("hedge_rate out of range 0..=0.5");
        }
        if !(0.0..=1.0).contains(&self.stigmergy_rate) {
            return err("stigmergy_rate out of range 0..=1");
        }
        if !(0.0..=1.0).contains(&self.stigmergy_gain) {
            return err("stigmergy_gain out of range 0..=1");
        }
        if !(0.5..=16.0).contains(&self.pheromone_cap) {
            return err("pheromone_cap out of range 0.5..=16");
        }
        if !(0.5..=0.999).contains(&self.pheromone_decay) {
            return err("pheromone_decay out of range 0.5..=0.999");
        }
        if !(0.0..=1.0).contains(&self.precision_gain) {
            return err("precision_gain out of range 0..=1");
        }
        if !(0.0..=1.0).contains(&self.lateral_inhibition) {
            return err("lateral_inhibition out of range 0..=1");
        }
        if self.quorum_min > 64 {
            return err("quorum_min out of range 0..=64");
        }
        if !(0.0..1.0).contains(&self.cooling) {
            return err("cooling factor must be within (0, 1)");
        }
        if self.explore_floor > self.explore_initial {
            return err("explore floor must not exceed initial temperature");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        assert!(EngineConfig::default().validate().is_ok());
    }

    #[test]
    fn o10_is_enforced() {
        let config = EngineConfig {
            contradiction_support_factor: 0.99, // cost 0.01 < gain 0.12
            ..EngineConfig::default()
        };
        assert!(config.validate().is_err());
    }
}
