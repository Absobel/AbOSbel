use crate::prelude::*;
use crate::print;
use crate::println;
use crate::vga_buffer::Color4b::*;
use crate::vga_buffer::*;
use crate::WRITER;

pub fn main() {
    WRITER!().clear();
    copypasta();
}

fn copypasta() {
    let copypasta = "Hey guys, did you know that in terms of male human and female Pokemon breeding, Vaporeon is the most compatible Pokemon for humans? Not only are they in the field egg group, which is mostly comprised of mammals, Vaporeon are an average of 3\"03' tall and 63.9 pounds, this means they're large enough to be able handle human dicks, and with their impressive Base Stats for HP and access to Acid Armor, you can be rough with one. Due to their mostly water based biology, there's no doubt in my mind that an aroused Vaporeon would be incredibly wet, so wet that you could easily have sex with one for hours without getting sore. They can also learn the moves Attract, Baby-Doll Eyes, Captivate, Charm, and Tail Whip, along with not having fur to hide nipples, so it'd be incredibly easy for one to get you in the mood. With their abilities Water Absorb and Hydration, they can easily recover from fatigue with enough water. No other Pokemon comes close to this level of compatibility. Also, fun fact, if you pull out enough, you can make your Vaporeon turn white. Vaporeon is literally built for human dick. Ungodly defense stat+high HP pool+Acid Armor means it can take cock all day, all shapes and sizes and still come for more";

    WRITER!().change_background_color(Color3b::Black);
    WRITER!().clear();

    const ALL_COLORS: [Color4b; 15] = [
        Blue, Green, Cyan, Red, Magenta, Brown, LightGray, DarkGray, LightBlue, LightGreen,
        LightCyan, LightRed, Pink, Yellow, White,
    ];

    for (i, b) in copypasta.chars().enumerate() {
        WRITER!().change_foreground_color(ALL_COLORS[i % 15]);
        print!("{b}")
    }
}

// DEBUG
#[allow(dead_code)]
pub fn hello_world() {
    println!("Hello World!");
}
