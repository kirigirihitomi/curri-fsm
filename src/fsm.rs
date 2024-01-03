use curri::{identity, if_else, Curry};
use std::collections::HashMap;

type State<T> = (Box<dyn Fn(T) -> T>, Box<dyn Fn(T) -> T>);

pub struct Machine<T> {
    pub context: T,
    pub current_state: String,
    pub states: HashMap<String, State<T>>,
    pub transitions: HashMap<String, Vec<(String, String)>>,
}

impl<T> Machine<T> {
    pub fn new(context: T, state: String) -> Self {
        Machine {
            context,
            current_state: state,
            states: HashMap::<String, State<T>>::new(),
            transitions: HashMap::<String, Vec<(String, String)>>::new(),
        }
    }
}

pub fn state<T, F, G>(name: &'static str, enter: F, exit: G) -> Box<dyn Fn(Machine<T>) -> Machine<T>>
where
    F: Fn(T) -> T + Copy + 'static,
    G: Fn(T) -> T + Copy + 'static,
{
    Box::new(move |mut machine: Machine<T>| {
        machine.states.insert(name.to_string(), (enter.curry(), exit.curry()));
        machine
    })
}

pub fn transitions<T>(on: &'static str, from: &'static str, to: &'static str) -> Box<dyn Fn(Machine<T>) -> Machine<T>> {
    Box::new(move |mut machine: Machine<T>| {
        machine
            .transitions
            .entry(on.to_string())
            .or_insert(Vec::new())
            .push((from.to_string(), to.to_string()));
        machine
    })
}

pub fn trigger<'a, T: 'a>(on: &'static str) -> Box<dyn Fn(Machine<T>) -> Machine<T> + 'a> {
    let check = |m: &Machine<_>| {
        let eq = |(from, _): &(String, _)| from == "*" || from == &m.current_state;
        let check = |transitions: &Vec<(String, String)>| transitions.iter().any(eq);
        m.transitions.get(on).map_or(false, check)
    };

    let if_fn = |mut m: Machine<_>| -> Machine<_> {
        let eq = |(from, _): &&(String, _)| from == "*" || from == &m.current_state;
        let transitions = m.transitions.get(on).unwrap();
        let (_, to) = transitions.iter().find(eq).unwrap();
        let (_, exit) = m.states.get(&m.current_state).unwrap();
        let (enter, _) = m.states.get(to).unwrap();
        let context = enter(exit(m.context));
        m.context = context;
        m.current_state = to.to_string();
        m
    };
    let else_fn = identity;
    if_else(check, if_fn, else_fn)
}

#[cfg(test)]
mod test {
    use super::*;
    use curri::*;

    #[test]
    fn test() {
        let machine = Machine {
            context: 0,
            states: HashMap::new(),
            current_state: "idle".to_string(),
            transitions: HashMap::new(),
        };

        let machine = compose!(
            state("idle", identity, |i: i32| i + 3),
            state("running", |i| i * 2, identity),
            state("paused", identity, identity),
            transitions("start", "idle", "running"),
            transitions("pause", "running", "paused"),
            transitions("resume", "paused", "running")
        )(machine);

        let start_trigger = trigger("start");
        let pause_trigger = trigger("pause");
        let resume_trigger = trigger("resume");

        let machine = start_trigger(machine);
        println!("{:?}, {:?}", machine.context, machine.current_state);
        assert_eq!(machine.current_state, "running");
        assert_eq!(machine.context, 6);

        let machine = pause_trigger(machine);
        println!("{:?}, {:?}", machine.context, machine.current_state);
        assert_eq!(machine.current_state, "paused");
        assert_eq!(machine.context, 6);

        let machine = resume_trigger(machine);
        println!("{:?}, {:?}", machine.context, machine.current_state);
        assert_eq!(machine.current_state, "running");
        assert_eq!(machine.context, 12);

        let machine = pause_trigger(machine);
        println!("{:?}, {:?}", machine.context, machine.current_state);
        assert_eq!(machine.current_state, "paused");
        assert_eq!(machine.context, 12);
    }
}
