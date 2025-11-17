use winit::event::KeyEvent;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceId {
    Winit(winit::event::DeviceId),
    Gilrs(gilrs::GamepadId),
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum DeviceEvent<'a> {
    Connected,
    Disconnected,
    Key(&'a KeyEvent),
    Button(&'a ButtonEvent),
}

#[derive(Debug, Clone, Copy)]
pub struct ButtonEvent {
    pub button: ButtonCode,
    pub value: f32,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonCode {
    LeftStickRight,
    LeftStickLeft,
    LeftStickUp,
    LeftStickDown,
    RightStickRight,
    RightStickLeft,
    RightStickUp,
    RightStickDown,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    South,
    East,
    North,
    West,
    LeftTrigger,
    LeftTrigger2,
    RightTrigger,
    RightTrigger2,
    Start,
    Select,
    LeftThumb,
    RightThumb,
    C,
    Z,
    Mode,
    Unknown,
}
