pub fn is_lead_surrogate(cp:u32) -> bool { (0xD800..=0xDBFF).contains(&cp) }

pub fn is_trail_surrogate(cp:u32) -> bool { (0xDC00..=0xDFFF).contains(&cp) }

pub fn combine_surrogate_pair(lead:u32, trail:u32) -> u32 {
	(lead - 0xD800) * 0x400 + trail - 0xDC00 + 0x10000
}
