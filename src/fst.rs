use std::fmt;

#[derive(Clone)]
struct Transition {
    from: usize,
    to: usize,
    on: Option<char>,
    with: Option<String>,
}

#[derive(Clone)]
pub struct FST {
    state_count: usize,
    accepting_states: Vec<usize>,
    transitions: Vec<Transition>,
}

fn shift_transitions(transitions: Vec<Transition>, shift: usize) -> Vec<Transition> {
    transitions.into_iter()
        .map(|t| Transition{
            from: t.from + shift,
            to: t.to + shift,
            on: t.on,
            with: t.with,
        })
        .collect()
}

fn shift_states(states: Vec<usize>, shift: usize) -> Vec<usize> {
    states.into_iter()
        .map(|s| s + shift)
        .collect()
}

impl FST {
    pub fn empty() -> Self {
        Self {
            state_count: 1,
            accepting_states: vec!(0),
            transitions: Vec::new(),
        }
    }

    pub fn all_reject() -> Self {
        Self {
            state_count: 1,
            accepting_states: Vec::new(),
            transitions: Vec::new(),
        }
    }

    pub fn symbol(symbol: char) -> Self {
        Self {
            state_count: 2,
            accepting_states: vec!(1),
            transitions: vec!(Transition{ from: 0, to: 1, on: Some(symbol), with: Some(symbol.to_string())}),
        }
    }

    pub fn string(s: &str) -> Self {
        let mut result = Self::empty();
        for c in s.chars() {
            result = FST::and(result, FST::symbol(c));
        }
        result
    }

    pub fn one_of_symbols(symbols: Vec<char>) -> Self {
        let mut result = Self::all_reject();
        for symbol in symbols {
            result = FST::or(result, FST::symbol(symbol));
        }
        result
    }

    pub fn consume(mut m: Self) -> Self {
        m.transitions = m.transitions.into_iter()
            .map(|mut t| { t.with = None; t })
            .collect();
        m
    }

    pub fn and(m1: Self, m2: Self) -> Self {
        let mut new_transitions = m1.transitions;
        new_transitions.extend(shift_transitions(m2.transitions, m1.state_count));
        for m1_accepting in m1.accepting_states {
            new_transitions.push(Transition{ from: m1_accepting, to: m1.state_count, on: None, with: None });
        }

        Self {
            state_count: m1.state_count + m2.state_count,
            accepting_states: shift_states(m2.accepting_states, m1.state_count),
            transitions: new_transitions,
        }
    }

    pub fn and_optionally(m1: Self, m2: Self) -> Self {
        let mut new_transitions = m1.transitions;
        new_transitions.extend(shift_transitions(m2.transitions, m1.state_count));
        for m1_accepting in &m1.accepting_states {
            new_transitions.push(Transition{ from: *m1_accepting, to: m1.state_count, on: None, with: None });
        }

        let mut new_accepting = m1.accepting_states;
        new_accepting.extend(shift_states(m2.accepting_states, m1.state_count));

        Self {
            state_count: m1.state_count + m2.state_count,
            accepting_states: new_accepting,
            transitions: new_transitions,
        }
    }

    pub fn wrap(m: Self, name: &str) -> Self {
        let mut new_transitions = shift_transitions(m.transitions, 2);
        new_transitions.push(Transition{ from: 0, to: 2, on: None, with: Some(name.to_string() + " ") });
        for m_accepting in shift_states(m.accepting_states, 2) {
            new_transitions.push(Transition{ from: m_accepting, to: 1, on: None, with: Some(" ".to_string()) });
        }

        Self {
            state_count: m.state_count + 2,
            accepting_states: vec!(1),
            transitions: new_transitions
        }
    }

    pub fn or(m1: Self, m2: Self) -> Self {
        let mut new_transitions = shift_transitions(m1.transitions, 1);
        new_transitions.extend(shift_transitions(m2.transitions, m1.state_count+1));
        new_transitions.push(Transition{ from: 0, to: 1, on: None, with: None });
        new_transitions.push(Transition{ from: 0, to: m1.state_count+1, on: None, with: None });

        let mut new_accepting = shift_states(m1.accepting_states, 1);
        new_accepting.extend(shift_states(m2.accepting_states, m1.state_count + 1));

        Self {
            state_count: m1.state_count + m2.state_count + 1,
            accepting_states: new_accepting,
            transitions: new_transitions,
        }
    }

    pub fn one_of(ms: Vec<Self>) -> Self {
        let mut result = Self::all_reject();
        for m in ms {
            result = Self::or(m, result);
        }
        result
    }

    pub fn at_least_once(mut m: Self) -> Self {
        for accepting in &m.accepting_states {
            m.transitions.push(Transition{ from: *accepting, to: 0, on: None, with: None });
        }
        m
    }

    pub fn repeated(mut m: Self) -> Self {
        m.transitions = shift_transitions(m.transitions, 1);
        m.accepting_states = vec!(0);
        for accepting in &m.accepting_states {
            m.transitions.push(Transition{ from: *accepting, to: 0, on: None, with: None });
        }
        m
    }
    
    fn symbol_step(&self, state: usize, path: Vec<String>, on: Option<char>) -> Vec<(usize, Vec<String>)> {
        self.transitions.iter()
            .filter(|t| t.from == state && t.on == on)
            .map(|t| {
                let mut new_path = path.clone();
                new_path.extend(t.with.clone());
                (t.to, new_path)
            })
            .collect()
    }

    fn epsilon_closure(&self, start_states: Vec<(usize, Vec<String>)>) -> Vec<(usize, Vec<String>)> {
        let mut result = Vec::new();
        let mut new_states = start_states;
        while {
            new_states = new_states.into_iter()
                .inspect(|s| result.push(s.clone()))
                .flat_map(|(state, path)| self.symbol_step(state, path, None))
                .collect();
            !new_states.is_empty()
        }{}
        result
    }

    fn step(&self, start_states: Vec<(usize, Vec<String>)>, symbol: char) -> Vec<(usize, Vec<String>)> {
        self.epsilon_closure(start_states.into_iter()
            .flat_map(|(state, path)| self.symbol_step(state, path, Some(symbol)))
            .collect())
    }

    pub fn match_string(&self, string: &str) -> Vec<Vec<String>> {
        let mut states = self.epsilon_closure(vec!((0, Vec::new())));
        for symbol in string.chars() {
            states = self.step(states, symbol);
            for (state, path) in &states {
                print!("{} {}; ", state, path.join(""));
            }
            println!();
        }

        states.into_iter()
            .filter(|(state, _)| self.accepting_states.contains(state))
            .map(|(_, path)| path)
            .collect()
    }
}

impl fmt::Display for FST {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} {}", self.state_count, self.transitions.len());
        for t in self.transitions.clone() {
            writeln!(f, "{}\t{}\t{}\t{}", t.from, t.to, t.on.unwrap_or('`'), t.with.unwrap_or_default());
        }

        for state in &self.accepting_states {
            write!(f, "{} ", state);
        }
        writeln!(f)
    }
}
