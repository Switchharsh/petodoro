//! What the pets say, per personality. Uppercase only (pixel font).

use crate::pets::Persona;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Hello,
    Pat,
    /// Pat once you're good friends.
    PatFriend,
    Snack,
    Full,
    FocusStart,
    FocusDone,
    BreakDone,
    Wake,
    Dizzy,
    Drop,
    Idle,
}

fn table(p: Persona, e: Event) -> &'static [&'static str] {
    use Event::*;
    use Persona::*;
    match (p, e) {
        (Sweet, Hello) => &["HI HI! LET'S DO OUR BEST!", "HELLO AGAIN ♥"],
        (Sweet, Pat) => &["HEHE ♥", "THAT TICKLES!", "YAY, PATS!"],
        (Sweet, PatFriend) => &["I LIKE YOU A LOT ♥", "BEST FRIENDS!"],
        (Sweet, Snack) => &["YUMMY! THANK YOU!", "NOM NOM ♥"],
        (Sweet, Full) => &["I'M SO FULL..."],
        (Sweet, FocusStart) => &["YOU'VE GOT THIS!", "LET'S FOCUS TOGETHER!"],
        (Sweet, FocusDone) => &["GREAT JOB! BREAK TIME ♥", "YOU DID IT!"],
        (Sweet, BreakDone) => &["READY? LET'S GO!", "BACK TO IT! WE CAN DO IT"],
        (Sweet, Wake) => &["OH! I'M AWAKE!", "*YAWN* HI!"],
        (Sweet, Dizzy) => &["THE ROOM IS SPINNING...", "@_@ WHOA"],
        (Sweet, Drop) => &["THANKS FOR THE RIDE!", "NICE SPOT!"],
        (Sweet, Idle) => &["DRINK SOME WATER!", "HOW ARE YOU DOING?", "DON'T FORGET TO STRETCH", "I'M RIGHT HERE ♥"],

        (Tsundere, Hello) => &["Y-YOU'RE BACK? WHATEVER.", "HMPH. TOOK YOU LONG ENOUGH"],
        (Tsundere, Pat) => &["B-BAKA! DON'T TOUCH ME!", "HMPH!", "IT'S NOT LIKE I LIKED IT!"],
        (Tsundere, PatFriend) => &["...YOU CAN KEEP GOING.", "...THANKS, I GUESS."],
        (Tsundere, Snack) => &["I-IT'S NOT LIKE I WANTED IT!", "...IT'S GOOD. HMPH."],
        (Tsundere, Full) => &["I'M NOT A PIG, BAKA!"],
        (Tsundere, FocusStart) => &["DON'T SLACK OFF, BAKA!", "FOCUS! I'M WATCHING."],
        (Tsundere, FocusDone) => &["NOT BAD... FOR YOU.", "F-FINE, TAKE A BREAK."],
        (Tsundere, BreakDone) => &["BREAK'S OVER, LAZYBONES!", "GET BACK TO WORK!"],
        (Tsundere, Wake) => &["I WASN'T SLEEPING!", "D-DON'T STARE AT ME!"],
        (Tsundere, Dizzy) => &["PUT ME DOWN, IDIOT!", "YOU'LL PAY FOR THAT..."],
        (Tsundere, Drop) => &["...I GUESS HERE IS OK.", "HMPH. FINE."],
        (Tsundere, Idle) => {
            &["IT'S NOT LIKE I'M WAITING.", "...ARE YOU DRINKING WATER?", "D-DON'T GET THE WRONG IDEA!"]
        }

        (Angry, Hello) => &["WHAT DO YOU WANT?!", "UGH. YOU AGAIN."],
        (Angry, Pat) => &["GRRR!", "HANDS OFF!", "YOU WANNA FIGHT?!"],
        (Angry, PatFriend) => &["...FINE. JUST THIS ONCE.", "GRR... OK, YOU'RE ALRIGHT."],
        (Angry, Snack) => &["...ACCEPTABLE.", "MORE. NOW."],
        (Angry, Full) => &["ENOUGH FOOD ALREADY!"],
        (Angry, FocusStart) => &["WORK. OR ELSE.", "NO PHONES! FOCUS!"],
        (Angry, FocusDone) => &["FINALLY. GO REST.", "HMPH. YOU SURVIVED."],
        (Angry, BreakDone) => &["BREAK'S OVER! MOVE IT!", "UP! WORK! NOW!"],
        (Angry, Wake) => &["WHO WOKE ME UP?!", "THIS BETTER BE GOOD."],
        (Angry, Dizzy) => &["STOP SHAKING ME!!", "I'LL REMEMBER THIS!"],
        (Angry, Drop) => &["DON'T MANHANDLE ME!", "TCH. FINALLY."],
        (Angry, Idle) => &["WHAT ARE YOU LOOKING AT?", "DRINK WATER. THAT'S AN ORDER.", "TCH."],

        (Sleepy, Hello) => &["MORNING... *YAWN*", "...HI... ZZZ"],
        (Sleepy, Pat) => &["MMM... NICE...", "...ZZ... HEHE"],
        (Sleepy, PatFriend) => &["YOU'RE COMFY... ♥"],
        (Sleepy, Snack) => &["MIDNIGHT SNACK...", "NOM... ZZZ"],
        (Sleepy, Full) => &["TOO FULL... NAP TIME"],
        (Sleepy, FocusStart) => &["OK... I'LL STAY AWAKE", "FOCUS... ZZ... FOCUS"],
        (Sleepy, FocusDone) => &["NAP BREAK? NAP BREAK.", "GOOD JOB... ZZZ"],
        (Sleepy, BreakDone) => &["FIVE MORE MINUTES...", "*YAWN* OK, OK..."],
        (Sleepy, Wake) => &["...FIVE MORE MINUTES", "WAS I ASLEEP...?"],
        (Sleepy, Dizzy) => &["SO... SPINNY...", "WHEEE... ZZZ"],
        (Sleepy, Drop) => &["COMFY SPOT... ZZZ", "...HERE IS GOOD"],
        (Sleepy, Idle) => &["*YAWN*", "SO SLEEPY...", "REST YOUR EYES TOO..."],

        (Hyper, Hello) => &["HIII!!! I MISSED YOU!!!", "LET'S GOOOO!!"],
        (Hyper, Pat) => &["AGAIN AGAIN AGAIN!!", "WHEEEE!!", "YAAAY!!"],
        (Hyper, PatFriend) => &["BEST FRIENDS FOREVER!!!"],
        (Hyper, Snack) => &["SNACK!!! NOM NOM NOM!", "MORE SUGAR!!!"],
        (Hyper, Full) => &["I'M FULL BUT HUNGRY!!"],
        (Hyper, FocusStart) => &["FOCUS MODE ACTIVATED!!", "LET'S CRUSH IT!!!"],
        (Hyper, FocusDone) => &["YOU DID IT!!! PARTY!!!", "BREAK!!! ZOOMIES!!"],
        (Hyper, BreakDone) => &["AAAND WE'RE BACK!!", "ONE MORE! YOU GOT THIS!!"],
        (Hyper, Wake) => &["I'M UP! I'M UP!!", "WHAT DID I MISS?!"],
        (Hyper, Dizzy) => &["AGAIN!!! THAT WAS FUN!!", "WHEEEEEE!!!"],
        (Hyper, Drop) => &["WHERE ARE WE?! COOL!!", "NEW SPOT!!!"],
        (Hyper, Idle) => &["HEY! HEY! HEY!", "DID YOU DRINK WATER?!", "WANNA DO A POMODORO?!", "SQUIRREL!!"],
    }
}

/// Pick a line; `r` is any random number.
pub fn pick(p: Persona, e: Event, r: u32) -> &'static str {
    let t = table(p, e);
    t[r as usize % t.len()]
}

/// The first line for an event (deterministic, for previews).
pub fn sample(p: Persona, e: Event) -> &'static str {
    table(p, e)[0]
}

/// Little wellbeing nudges for break notifications.
pub const BREAK_TIPS: [&str; 8] = [
    "Drink a glass of water.",
    "Stand up and stretch your arms.",
    "Look at something 20 feet away for 20 seconds.",
    "Take five slow, deep breaths.",
    "Roll your shoulders and unclench your jaw.",
    "Grab a healthy snack.",
    "Walk around for a minute.",
    "Rest your eyes - close them for a bit.",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::has_glyph;

    #[test]
    fn every_line_is_drawable() {
        let personas = [Persona::Sweet, Persona::Tsundere, Persona::Angry, Persona::Sleepy, Persona::Hyper];
        let events = [
            Event::Hello,
            Event::Pat,
            Event::PatFriend,
            Event::Snack,
            Event::Full,
            Event::FocusStart,
            Event::FocusDone,
            Event::BreakDone,
            Event::Wake,
            Event::Dizzy,
            Event::Drop,
            Event::Idle,
        ];
        for p in personas {
            for e in events {
                for line in table(p, e) {
                    assert!(line.len() <= 32, "{line} is too long");
                    for c in line.chars() {
                        assert!(c == ' ' || has_glyph(c), "{c:?} in {line:?} has no glyph");
                    }
                }
            }
        }
    }
}
