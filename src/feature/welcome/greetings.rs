//! 40 welcome greetings - English, randomized (Javadoc style).

/// Returns a random greeting for the given display name.
/// Uses thread-local RNG, no `sleep` - pure sync.
pub fn random_greeting(name: &str) -> String {
    const GREETINGS: [&str; 40] = [
        "Welcome to **Loki Dev**, {user} - glad you're here! 🎉",
        "Hey {user}, welcome aboard! Check out <#1509503661431197736> and get verified in <#1509503655487737916>.",
        "Hello {user}! You're now part of Loki Dev - explore <#1527976303284715610> to pick your language.",
        "Welcome, {user}! 🚀 Dive into <#1509503673988808785> and share your ideas.",
        "Hi {user}, welcome to the community! Don't forget to read the rules 📕",
        "Yo {user}! Welcome - pick your language in 🌐｜roles and start chatting!",
        "{user} just joined! Welcome to Loki Dev - make yourself at home 🏠",
        "Welcome {user}! Need help? Open a ticket in <#1509503682914549861>.",
        "Hey there {user}! Welcome to Loki Dev - where code meets community 💻",
        "Welcome aboard, {user}! Check <#1509503667332714566> for updates.",
        "Hi {user}! Glad you found us - enjoy your stay! ✨",
        "Welcome {user}! Don't be shy - say hi in <#1509503673988808785>",
        "Hello {user}, welcome! Your journey at Loki Dev starts now 🌟",
        "Hey {user}! Welcome - grab your roles in 🌐｜roles and chat in your language!",
        "Welcome {user}! 🚀 We're happy to have you here.",
        "Hi {user}, welcome to Loki Dev! Explore, learn, build 🔧",
        "Welcome, {user}! Feel free to ask questions - we're friendly 😊",
        "Hey {user}! Welcome to our dev corner - enjoy! 🎨",
        "Hello {user}! Welcome - check #rules then chat away 💬",
        "Welcome {user}! 🎉 Thanks for joining Loki Dev!",
        "Hi {user}! Welcome - your next big idea starts here 💡",
        "Hey {user}, welcome! Don't forget to verify ✅ to unlock channels.",
        "Welcome {user}! 🌈 Diversity and code - that's us!",
        "Hello {user}! Welcome to Loki Dev - let's build together 🛠️",
        "Hi {user}! Welcome - see <#1509503680678723614> for ideas",
        "Welcome {user}! Check out our GitHub log <#1509503672197840996> 🧠",
        "Hey {user}! Welcome aboard - grab a coffee ☕ and chat!",
        "Welcome {user}! 🚀 Ready to code?",
        "Hi {user}, welcome! We're excited to have you 🎊",
        "Hello {user}! Welcome to Loki Dev - enjoy the community 🤝",
        "Welcome {user}! Don't miss <#1509503667332714566> announcements 📢",
        "Hey {user}! Welcome - pick EN/RU in 🌐｜roles for bot language!",
        "Welcome {user}! 🌟 New member, new ideas!",
        "Hi {user}! Welcome to Loki Dev - stay, chat, create 🎨",
        "Hello {user}! Welcome - need a voice? Join 🔊｜work-room-1",
        "Welcome {user}! 🎉 Let's make something awesome!",
        "Hey {user}! Welcome - check #welcome and get started!",
        "Welcome {user}! Your presence makes us stronger 💪",
        "Hi {user}! Welcome to Loki Dev - where devs meet 👋",
        "Hello {user}, welcome! Enjoy your stay - and verify to unlock! ✅",
    ];
    let idx = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        format!(
            "{}-{}",
            name,
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        )
        .hash(&mut h);
        (h.finish() as usize) % GREETINGS.len()
    };
    GREETINGS[idx].replace("{user}", name)
}
