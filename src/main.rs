use std::{thread, time};

use enigo::{
    Direction::{Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use gilrs::{Axis, EventType, Gilrs};

static AXIS_THRESHOLD: f32 = 0.5;
const DEBUG: bool = false;
const SLEEP_MS: u64 = 40;

fn main() {
    // apt install libxdo-dev
    // pad codes are from 0 to 9
    // 0,1     START, SELECT   - buttons
    // 2,3,4,5 DU, DD, DL, DR - left d-pad buttons
    // 2,3,4,5 XU, XD, XL, XR - left stick axises
    // 6,7,8,9 YU, YD, YL, YR - left stick axises
    // 6,7,8,9 BA, BB, BX, BY - buttons A,B,X,Y

    // joypad 0 - letters from 0 to 9
    let pad_0_chars: Vec<char> = vec!['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'];
    let pad_1_chars: Vec<char> = vec!['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'];
    let pad_2_chars: Vec<char> = vec!['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';'];
    let pad_3_chars: Vec<char> = vec!['z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/'];
    let pads_chars = [pad_0_chars, pad_1_chars, pad_2_chars, pad_3_chars];
    let mut ascii_state: Vec<bool> = vec![false; 128];

    //joystick event lib
    let mut gilrs = Gilrs::new().unwrap();
    // keyboard emulator lib
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    // Iterate over all connected gamepads
    print_controllers(&gilrs);

    loop {
        // Examine new events
        while let Some(event) = gilrs.next_event() {
            if DEBUG {
                match event.event {
                    EventType::ButtonPressed(_, code)
                    | EventType::ButtonRepeated(_, code)
                    | EventType::ButtonReleased(_, code) => {
                        println!("pad {}: button {:?}", event.id, code.into_u32(),)
                    }
                    EventType::ButtonChanged(_, _, code) => {
                        println!("pad {}: button {:?}", event.id, code.into_u32(),)
                    }
                    EventType::AxisChanged(axis, _, code) => {
                        println!("pad {}: axis {:?} {:?}", event.id, code.into_u32(), axis)
                    }
                    _ => {}
                }
            }
            // println!("pad {}: {:?}", event.id, event.event);

            let mut buttons: Vec<(bool, u32)> = Vec::with_capacity(8);

            match event.event {
                EventType::Connected | EventType::Disconnected | EventType::Dropped => {
                    print_controllers(&gilrs);
                    continue;
                }

                // button press and release
                EventType::ButtonPressed(_, code) => buttons.push((true, code.into_u32())),
                EventType::ButtonReleased(_, code) => buttons.push((false, code.into_u32())),

                //axis change
                EventType::AxisChanged(axis, offset, _) => {
                    match axis {
                        // d-up
                        Axis::LeftStickY => {
                            if axis_to_buttons(&mut buttons, offset, 66080, 66081) {
                                continue;
                            }
                        }
                        // d-left
                        Axis::LeftStickX | Axis::LeftZ => {
                            if axis_to_buttons(&mut buttons, offset, 66083, 66082) {
                                continue;
                            }
                        }
                        // b-left
                        Axis::RightStickX => {
                            if axis_to_buttons(&mut buttons, offset, 65826, 65825) {
                                continue;
                            }
                        }
                        // b-up
                        Axis::RightZ | Axis::RightStickY => {
                            if axis_to_buttons(&mut buttons, offset, 65827, 65824) {
                                continue;
                            }
                        }
                        _ => continue,
                    }
                }
                _ => continue,
            };

            for (pad_set, pad_code) in &buttons {
                let pad_letter: usize = if *pad_code > 0 {
                    match pad_code {
                        // 0,1     START, SELECT  - buttons
                        65832 => 0,
                        65850 => 0,
                        65848 => 0,
                        65833 => 1,
                        65851 => 1,
                        65849 => 1,

                        // 2,3,4,5 DU, DD, DL, DR - left d-pad buttons
                        // 2,3,4,5 XU, XD, XL, XR - left stick axises
                        66080 => 2,
                        66081 => 3,
                        66082 => 4,
                        66083 => 5,

                        // 6,7,8,9 YU, YD, YL, YR - left stick axises
                        // 6,7,8,9 BA, BB, BX, BY - buttons A,B,X,Y
                        65825 => 6,
                        65826 => 7,
                        65824 => 8,
                        65827 => 9,
                        65841 => 6,
                        65840 => 7,
                        65844 => 8,
                        65843 => 9,

                        _ => continue,
                    }
                } else {
                    continue;
                };

                let pad_id: usize = event.id.into();
                button_to_keyboard(
                    pad_id,
                    *pad_set,
                    pad_letter,
                    &mut enigo,
                    &pads_chars,
                    &mut ascii_state,
                );
            }
            buttons.clear();
            //sleep for 50 ms for cpu relax
            thread::sleep(time::Duration::from_millis(SLEEP_MS));
        }
    }
}

fn print_controllers(gilrs: &Gilrs) {
    println!();
    println!("----------------------------------------------------------");
    for (id, gamepad) in gilrs.gamepads() {
        println!("{} : {} {:?}", id, gamepad.name(), gamepad.power_info());
    }
    println!("----------------------------------------------------------");
}

fn axis_to_buttons(
    buttons: &mut Vec<(bool, u32)>,
    offset: f32,
    positive_code: u32,
    negative_code: u32,
) -> bool {
    if offset > 0.1 {
        let press = offset > AXIS_THRESHOLD;
        if press {
            buttons.push((false, negative_code));
        }
        buttons.push((press, positive_code));
    } else if offset < 0.1 {
        let press = offset < -AXIS_THRESHOLD;
        if press {
            buttons.push((false, positive_code));
        }
        buttons.push((press, negative_code));
    } else {
        return true;
    }
    false
}

fn button_to_keyboard(
    pad_id: usize,
    pad_set: bool,
    pad_letter: usize,
    enigo: &mut Enigo,
    pads_chars: &[Vec<char>; 4],
    ascii_state: &mut [bool],
) {
    let letters = &pads_chars[pad_id];
    let letter = letters[pad_letter];
    let old_state = ascii_state[letter as usize];
    if pad_set {
        if !old_state {
            let _ = enigo.key(Key::Unicode(letter), Press);
            ascii_state[letter as usize] = true;
        }
    } else if old_state {
        let _ = enigo.key(Key::Unicode(letter), Release);
        ascii_state[letter as usize] = false;
    }
}
