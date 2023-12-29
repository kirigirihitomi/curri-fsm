use curri::compose_vec;

use crate::fsm::*;
use std::ffi::*;

type Def = dyn Fn(Machine<*const c_void>) -> Machine<*const c_void>;

#[allow(non_snake_case)]
pub extern "C" fn CurriMachine(context: *const c_void, default: *const c_char) -> *const c_void {
    let default = unsafe { CStr::from_ptr(default).to_str().unwrap() };
    let machine = Machine::<*const c_void>::new(context, default.to_string());
    Box::into_raw(Box::new(machine)) as *const c_void
}

#[allow(non_snake_case)]
pub extern "C" fn CurriDropMachine(machine: *const c_void) {
    unsafe {
        let _ = Box::from_raw(machine as *mut Machine<*const c_void>);
    }
}

#[allow(non_snake_case)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn CurriState(
    name: *const c_char, enter: extern "C" fn(*const c_void) -> *const c_void, exit: extern "C" fn(*const c_void) -> *const c_void,
) -> *mut Def {
    let name = unsafe { CStr::from_ptr(name).to_str().unwrap() };
    let enter = move |context: *const c_void| -> *const c_void { enter(context) };
    let exit = move |context: *const c_void| -> *const c_void { exit(context) };
    let state = state(name, enter, exit);
    Box::into_raw(state)
}

#[allow(non_snake_case)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn CurriTransitions(on: *const c_char, from: *const c_char, to: *const c_char) -> *mut Def {
    let on = unsafe { CStr::from_ptr(on).to_str().unwrap() };
    let from = unsafe { CStr::from_ptr(from).to_str().unwrap() };
    let to = unsafe { CStr::from_ptr(to).to_str().unwrap() };
    let transitions: Box<dyn Fn(Machine<*const c_void>) -> Machine<*const c_void>> = transitions(on, from, to);
    Box::into_raw(transitions)
}

#[allow(non_snake_case)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn CurriTrigger(on: *const c_char) -> *mut Def {
    let on = unsafe { CStr::from_ptr(on).to_str().unwrap() };
    let trigger: Box<dyn Fn(Machine<*const c_void>) -> Machine<*const c_void>> = trigger(on);
    Box::into_raw(trigger)
}

#[allow(non_snake_case)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn CurriCompose(f: *mut *mut Def, l: c_int) -> *mut Def {
    let mut states = Vec::new();
    for i in 0..l {
        let state = unsafe { Box::from_raw(*f.offset(i as isize)) };
        states.push(state);
    }
    let compose: Box<dyn Fn(Machine<*const c_void>) -> Machine<*const c_void>> = compose_vec(states);
    Box::into_raw(compose)
}

#[allow(non_snake_case)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn CurriRun(act: *mut Def, machine: *const c_void) -> *const c_void {
    let machine = unsafe { Box::from_raw(machine as *mut Machine<*const c_void>) };
    let act = unsafe { Box::from_raw(act) };
    // let act = unsafe { Box::from_raw(act) };
    let machine = Box::new(act(*machine));
    Box::into_raw(machine) as *const c_void
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::raw::c_char;
    use std::ptr;

    extern "C" fn identity(context: *const c_void) -> *const c_void {
        println!("identity: {:?}", context);
        context
    }

    #[test]
    fn test_machine() {
        let context = (&mut 199) as *mut i32 as *const c_void;
        let default = CString::new("default").unwrap();
        let default_ptr = default.as_ptr() as *const c_char;

        let machine = CurriMachine(context, default_ptr);
        let machine = unsafe { Box::from_raw(machine as *mut Machine<*const c_void>) };
        assert_eq!(unsafe { *(machine.context as *const i32) }, 199);
    }

    #[test]
    fn test_drop() {
        let context = ptr::null();
        let default = CString::new("default").unwrap();
        let default_ptr = default.as_ptr() as *const c_char;

        let machine = CurriMachine(context, default_ptr);
        CurriDropMachine(machine);
    }

    #[test]
    fn test_compose() {
        let context = ptr::null();
        let default = CString::new("state1").unwrap();
        let default_ptr = default.as_ptr() as *const c_char;

        let machine = CurriMachine(context, default_ptr);

        let state1 = CString::new("state1").unwrap();
        let state1_ptr = state1.as_ptr() as *const c_char;
        let state2 = CString::new("state2").unwrap();
        let state2_ptr = state2.as_ptr() as *const c_char;
        let state3 = CString::new("state3").unwrap();
        let state3_ptr = state3.as_ptr() as *const c_char;

        let state1 = CurriState(state1_ptr, identity, identity);
        let state2 = CurriState(state2_ptr, identity, identity);
        let state3 = CurriState(state3_ptr, identity, identity);

        let raw_array = [state1, state2, state3];
        let as_ptr = raw_array.as_ptr() as *mut *mut Def;
        let com = CurriCompose(as_ptr, 3);
        let machine = CurriRun(com, machine);

        let transition = CString::new("transition").unwrap();
        let transition_ptr = transition.as_ptr() as *const c_char;

        let transition1 = CurriTransitions(transition_ptr, state1_ptr, state2_ptr);
        let transition2 = CurriTransitions(transition_ptr, state2_ptr, state3_ptr);
        let transition3 = CurriTransitions(transition_ptr, state3_ptr, state1_ptr);

        let raw_array = [transition1, transition2, transition3];
        let as_ptr = raw_array.as_ptr() as *mut *mut Def;
        let com = CurriCompose(as_ptr, 3);
        let machine = CurriRun(com, machine);

        let trigger = CurriTrigger(transition_ptr);
        let machine = CurriRun(trigger, machine);
        let machine = unsafe { Box::from_raw(machine as *mut Machine<*const c_void>) };
        assert_eq!(machine.current_state, "state2");
        let machine = Box::into_raw(machine) as *const c_void;
        let trigger = CurriTrigger(transition_ptr);
        let machine = CurriRun(trigger, machine);
        let machine = unsafe { Box::from_raw(machine as *mut Machine<*const c_void>) };
        assert_eq!(machine.current_state, "state3");
        let machine = Box::into_raw(machine) as *const c_void;
        let trigger = CurriTrigger(transition_ptr);
        let machine = CurriRun(trigger, machine);
        let machine = unsafe { Box::from_raw(machine as *mut Machine<*const c_void>) };
        assert_eq!(machine.current_state, "state1");
    }
}
