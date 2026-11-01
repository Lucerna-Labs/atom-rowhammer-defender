use super::RegionIdentity;

const INDEX_BITS: usize = 22;
const INDEX_COUNT: usize = 1 << INDEX_BITS;
const WORDS: usize = INDEX_COUNT / 64;
const EPOCH_CAP: u64 = 262_144;

pub(super) struct SketchObservation {
    pub(super) renewed: bool,
    pub(super) lead_class: Option<u8>,
}

pub(super) struct SketchThermalChannel {
    bits: [Vec<u64>; 3],
    count: u64,
    heat: f64,
    last_time_ns: Option<u64>,
    lead_level: u64,
}

impl SketchThermalChannel {
    pub(super) fn new() -> Self {
        Self {
            bits: std::array::from_fn(|_| vec![0; WORDS]),
            count: 0,
            heat: 0.0,
            last_time_ns: None,
            lead_level: 0,
        }
    }

    fn clear_epoch(&mut self) {
        for words in &mut self.bits {
            words.fill(0);
        }
        self.count = 0;
        self.heat = 0.0;
        self.lead_level = 0;
    }

    pub(super) fn observe(&mut self, identity: RegionIdentity, time_ns: u64) -> SketchObservation {
        let mut renewed = false;
        if self.count >= EPOCH_CAP || self.last_time_ns.is_some_and(|last| time_ns < last) {
            self.clear_epoch();
            renewed = true;
        }
        if let Some(last) = self.last_time_ns {
            let previous_heat = self.heat;
            self.heat *= (-((time_ns - last) as f64) / 500_000.0).exp();
            if previous_heat >= 1.0 && self.heat < 1.0 {
                self.lead_level = 0;
                renewed = true;
            }
        }
        self.last_time_ns = Some(time_ns);

        let value =
            identity.block ^ (u64::from(identity.pid) << 32) ^ ((identity.kind as u64) << 60);
        let seeds = [
            0x1234_5678_9abc_def0,
            0x0fed_cba9_8765_4321,
            0xa5a5_5a5a_c3c3_3c3c,
        ];
        let indexes = seeds.map(|seed| (mix(value ^ seed) as usize) & (INDEX_COUNT - 1));
        let seen = indexes
            .iter()
            .enumerate()
            .all(|(index, bit)| self.bits[index][bit / 64] & (1_u64 << (bit % 64)) != 0);
        if seen {
            self.heat += 1.0;
        }
        for (index, bit) in indexes.into_iter().enumerate() {
            self.bits[index][bit / 64] |= 1_u64 << (bit % 64);
        }
        self.count += 1;

        let level = self.heat.floor().max(0.0) as u64;
        let lead_class = if level >= 122 && level > self.lead_level {
            self.lead_level = level;
            Some(7)
        } else {
            None
        };
        SketchObservation {
            renewed,
            lead_class,
        }
    }
}

fn mix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
