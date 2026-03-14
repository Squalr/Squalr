use uuid::Uuid;

pub const STARTUP_MESSAGES: &[&str] = &[
    "Squalr stretched, yawned, and is ready to scan.",
    "Tiny gremlins aligned the pointers. Probably.",
    "Memory snacks acquired. Hunting begins.",
    "Squalr is awake and politely judging your target process.",
    "Warm caches, sharp teeth, excellent day for reverse engineering.",
    "The scan goblin has reported for duty.",
    "Squalr did a little startup wiggle and is now operational.",
    "Fresh boot, clear eyes, suspicious addresses everywhere.",
    "All systems nominal. Mischief optional.",
    "Squalr brought extra buckets for all these bytes.",
    "The engine purred. The memory map did not object.",
    "Startup complete. Time to sniff around some RAM.",
];

/// Picks a random startup message from the engine message bank.
pub fn get_random_startup_message() -> &'static str {
    if STARTUP_MESSAGES.is_empty() {
        return "Squalr started, but the cute message bank was empty.";
    }

    let random_value = u128::from_le_bytes(*Uuid::new_v4().as_bytes());
    let message_index = select_startup_message_index(random_value, STARTUP_MESSAGES.len());

    STARTUP_MESSAGES[message_index]
}

fn select_startup_message_index(
    random_value: u128,
    message_count: usize,
) -> usize {
    if message_count == 0 {
        return 0;
    }

    (random_value % message_count as u128) as usize
}

#[cfg(test)]
mod tests {
    use super::{STARTUP_MESSAGES, get_random_startup_message, select_startup_message_index};

    #[test]
    fn startup_message_bank_is_not_empty() {
        assert!(!STARTUP_MESSAGES.is_empty());
    }

    #[test]
    fn selected_index_stays_within_message_bank_bounds() {
        let message_index = select_startup_message_index(u128::MAX, STARTUP_MESSAGES.len());

        assert!(message_index < STARTUP_MESSAGES.len());
    }

    #[test]
    fn random_startup_message_is_from_message_bank() {
        let startup_message = get_random_startup_message();

        assert!(STARTUP_MESSAGES.contains(&startup_message));
    }
}
