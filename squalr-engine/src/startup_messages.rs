use uuid::Uuid;

pub const STARTUP_MESSAGES: &[&str] = &[
    "\"Mute the nitpickers, block the outraged, like the kind, follow the insightful.\" - Naval Ravikant",
    "\"Desire is a contract that you make with yourself to be unhappy until you get what you want.\" - Naval Ravikant",
    "\"Seek wealth, not money or status. Wealth is having assets that earn while you sleep. Money is how we transfer time and wealth. Status is your place in the social hierarchy.\" - Naval Ravikant",
    "\"Understand that ethical wealth creation is possible. If you secretly despise wealth, it will elude you.\" - Naval Ravikant",
    "\"Ignore people playing status games. They gain status by attacking people playing wealth creation games.\" - Naval Ravikant",
    "\"You're not going to get rich renting out your time. You must own equity - a piece of a business - to gain your financial freedom.\" - Naval Ravikant",
    "\"You will get rich by giving society what it wants but does not yet know how to get. At scale.\" - Naval Ravikant",
    "\"Pick an industry where you can play long term games with long term people.\" - Naval Ravikant",
    "\"Play iterated games. All the returns in life, whether in wealth, relationships, or knowledge, come from compound interest.\" - Naval Ravikant",
    "\"Pick business partners with high intelligence, energy, and, above all, integrity.\" - Naval Ravikant",
    "\"Don't partner with cynics and pessimists. Their beliefs are self-fulfilling.\" - Naval Ravikant",
    "\"Learn to sell. Learn to build. If you can do both, you will be unstoppable.\" - Naval Ravikant",
    "\"Arm yourself with specific knowledge, accountability, and leverage.\" - Naval Ravikant",
    "\"Specific knowledge is knowledge that you cannot be trained for. If society can train you, it can train someone else, and replace you.\" - Naval Ravikant",
    "\"Specific knowledge is found by pursuing your genuine curiosity and passion rather than whatever is hot right now.\" - Naval Ravikant",
    "\"Building specific knowledge will feel like play to you but will look like work to others.\" - Naval Ravikant",
    "\"When specific knowledge is taught, it's through apprenticeships, not schools.\" - Naval Ravikant",
    "\"Specific knowledge is often highly technical or creative. It cannot be outsourced or automated.\" - Naval Ravikant",
    "\"Embrace accountability, and take business risks under your own name. Society will reward you with responsibility, equity, and leverage.\" - Naval Ravikant",
    "\"The most accountable people have singular, public, and risky brands: Oprah, Trump, Kanye, Elon.\" - Naval Ravikant",
    "\"Fortunes require leverage. Business leverage comes from capital, people, and products with no marginal cost of replication (code and media).\" - Naval Ravikant",
    "\"Capital means money. To raise money, apply your specific knowledge, with accountability, and show resulting good judgment.\" - Naval Ravikant",
    "\"Labor means people working for you. It's the oldest and most fought-over form of leverage. Labor leverage will impress your parents, but don't waste your life chasing it.\" - Naval Ravikant",
    "\"Capital and labor are permissioned leverage. Everyone is chasing capital, but someone has to give it to you. Everyone is trying to lead, but someone has to follow you.\" - Naval Ravikant",
    "\"Code and media are permissionless leverage. They're the leverage behind the newly rich. You can create software and media that works for you while you sleep.\" - Naval Ravikant",
    "\"An army of robots is freely available - it's just packed in data centers for heat and space efficiency. Use it.\" - Naval Ravikant",
    "\"If you can't code, write books and blogs, record videos and podcasts.\" - Naval Ravikant",
    "\"Leverage is a force multiplier for your judgement.\" - Naval Ravikant",
    "\"Judgement requires experience, but can be built faster by learning foundational skills.\" - Naval Ravikant",
    "\"There is no skill called \"business\". Avoid business magazines and business classes.\" - Naval Ravikant",
    "\"Study microeconomics, game theory, psychology, persuasion, ethics, mathematics, and computers.\" - Naval Ravikant",
    "\"Reading is faster than listening. Doing is faster than watching.\" - Naval Ravikant",
    "\"You should be too busy to \"do coffee,\" while still keeping an uncluttered calendar.\" - Naval Ravikant",
    "\"Set and enforce an aspirational personal hourly rate. If fixing a problem will save less than your hourly rate, ignore it. If outsourcing a task will cost less than your hourly rate, outsource it.\" - Naval Ravikant",
    "\"Work as hard as you can. Even though who you work with and what you work on are more important than how hard you work.\" - Naval Ravikant",
    "\"Become the best in the world at what you do. Keep redefining what you do until this is true.\" - Naval Ravikant",
    "\"There are no get rich quick schemes. That's just someone else getting rich off you.\" - Naval Ravikant",
    "\"Apply specific knowledge, with leverage, and eventually you will get what you deserve.\" - Naval Ravikant",
    "\"When you're finally wealthy, you'll realize that it wasn't what you were seeking in the first place. But that's for another day.\" - Naval Ravikant",
    "\"Brilliant thinking is rare, but courage is in even shorter supply than genius.\" - Peter Thiel",
    "\"And they went out over the breadth of the earth, and compassed the camp of the saints about, and the beloved city. And fire came down from God out of heaven, and devoured them.\" - Revelation 20:9",
    "\"Your universe has no meaning to them. They will not try to understand. They will be tired, they will be cold, they will make a fire with your beautiful oak door....\" - Jean Raspail, The Camp of the Saints",
    "\"I wish it need not have happened in my time,\" said Frodo. \"So do I,\" said Gandalf, \"and so do all who live to see such times. But that is not for them to decide. All we have to decide is what to do with the time that is given us.\" - J.R.R. Tolkien, The Fellowship of the Ring",
    "\"Give me a lever long enough, and a place to stand, and I will move the earth.\" - Archimedes",
    "\"The Supreme Court has made its decision, now let them enforce it.\" - Andrew Jackson",
    "\"Never interrupt your enemy when he is making a mistake.\" - Napoleon Bonaparte",
    "\"Men are Moved by two levers only: fear and self interest.\" - Napoleon Bonaparte",
    "\"I saw the crown of France laying on the ground, so I picked it up with my sword.\" - Napoleon Bonaparte",
    "\"If the rule you followed brought you to this, of what use was the rule?\" - Anton Chigurh, No Country for Old Men",
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
