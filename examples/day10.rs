use {
    aoc_2022::*,
    clap::Parser,
    std::{
        fmt::{Debug, Formatter, Result as FmtResult},
        num::{NonZeroU32, ParseIntError},
        slice::Iter,
        str::{FromStr, Split},
    },
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Instruction {
    Noop,
    AddX(i32),
}

impl Instruction {
    const fn cycles(self) -> NonZeroU32 {
        match self {
            Self::AddX(_) =>
            // SAFETY: 2 is non-zero
            unsafe { NonZeroU32::new_unchecked(2_u32) },
            Self::Noop =>
            // SAFETY: 1 is non-zero
            unsafe { NonZeroU32::new_unchecked(1_u32) },
        }
    }

    fn finish(self, cpu_state: &mut CpuState) {
        match self {
            Self::AddX(v) => cpu_state.x += v,
            Self::Noop => {}
        }
    }
}

#[derive(Debug, PartialEq)]
enum InstructionParseError<'s> {
    NoInitialToken,
    InvalidInstructionToken(&'s str),
    NoValueToken,
    FailedToParseValue(ParseIntError),
    ExtraTokenFound(&'s str),
}

impl<'s> TryFrom<&'s str> for Instruction {
    type Error = InstructionParseError<'s>;

    fn try_from(instruction_str: &'s str) -> Result<Self, Self::Error> {
        use InstructionParseError as Error;

        let mut instruction_token_iter: Split<char> = instruction_str.split(' ');

        let instruction: Self = match instruction_token_iter.next() {
            None => Err(Error::NoInitialToken),
            Some("addx") => match instruction_token_iter.next() {
                None => Err(Error::NoValueToken),
                Some(value_str) => i32::from_str(value_str)
                    .map(Self::AddX)
                    .map_err(Error::FailedToParseValue),
            },
            Some("noop") => Ok(Self::Noop),
            Some(invalid_instruction_str) => {
                Err(Error::InvalidInstructionToken(invalid_instruction_str))
            }
        }?;

        match instruction_token_iter.next() {
            Some(extra_token) => Err(Error::ExtraTokenFound(extra_token)),
            None => Ok(instruction),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Instructions(Vec<Instruction>);

impl Instructions {
    fn iter(&self) -> Iter<Instruction> {
        self.0.iter()
    }
}

impl<'s> TryFrom<&'s str> for Instructions {
    type Error = InstructionParseError<'s>;

    fn try_from(instructions_str: &'s str) -> Result<Self, Self::Error> {
        let mut instructions: Self = Self(Vec::new());

        for instruction_str in instructions_str.split('\n') {
            instructions.0.push(instruction_str.try_into()?);
        }

        Ok(instructions)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CpuState {
    cycle: u32,
    x: i32,
}

impl CpuState {
    fn execute<'cs, I: Iterator<Item = &'cs Instruction>>(
        &'cs mut self,
        instruction_iter: I,
    ) -> CpuStateIter<'cs, I> {
        let mut cpu_state_iter: CpuStateIter<I> = CpuStateIter::<I> {
            cpu_state: self,
            instruction_iter,
            current_instruction: None,
            instruction_cycles: 0_u32,
        };

        cpu_state_iter.pre_cycle();

        cpu_state_iter
    }

    #[inline]
    fn signal_strength(self) -> i32 {
        self.cycle as i32 * self.x
    }

    fn cycle_mod_40_is_20(&self) -> bool {
        self.cycle % 40_u32 == 20_u32
    }
}

impl Default for CpuState {
    fn default() -> Self {
        Self {
            cycle: 1_u32,
            x: 1_i32,
        }
    }
}

struct CpuStateIter<'cs, I: Iterator<Item = &'cs Instruction>> {
    cpu_state: &'cs mut CpuState,
    instruction_iter: I,
    current_instruction: Option<Instruction>,
    instruction_cycles: u32,
}

impl<'cs, I: Iterator<Item = &'cs Instruction>> CpuStateIter<'cs, I> {
    /// Run the cycle before the state is considered readable
    ///
    /// For each call to `next`, this must have been called once prior
    fn pre_cycle(&mut self) {
        if self.current_instruction.is_none() {
            if let Some(instruction) = self.instruction_iter.next() {
                self.instruction_cycles = instruction.cycles().get();
                self.current_instruction = Some(*instruction);
            }
        }
    }

    /// Run the cycle after the state is considered readable
    ///
    /// For each call to `next`, this must be called once after
    fn post_cycle(&mut self) {
        self.cpu_state.cycle += 1_u32;
        self.instruction_cycles -= 1_u32;

        if self.instruction_cycles == 0_u32 {
            if let Some(current_instruction) = self.current_instruction.take() {
                current_instruction.finish(self.cpu_state);
            } else {
                eprintln!("`CpuStateIter::cycle` called with no current instruction");
            }
        }
    }
}

impl<'cs, I: Iterator<Item = &'cs Instruction>> Debug for CpuStateIter<'cs, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("CpuStateIter")
            .field("cpu_state", self.cpu_state)
            .field("current_instruction", &self.current_instruction)
            .field("instruction_cycles", &self.instruction_cycles)
            .finish()
    }
}

impl<'cs, I: Iterator<Item = &'cs Instruction>> Iterator for CpuStateIter<'cs, I> {
    type Item = CpuState;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_instruction.is_some() {
            let cpu_state: Option<CpuState> = Some(*self.cpu_state);

            self.post_cycle();
            self.pre_cycle();

            cpu_state
        } else {
            None
        }
    }
}

fn main() {
    let args: Args = Args::parse();
    let input_file_path: &str = args.input_file_path("input/day10.txt");

    if let Err(err) =
        // SAFETY: This operation is unsafe, we're just hoping nobody else touches the file while
        // this program is executing
        unsafe {
            open_utf8_file(
                input_file_path,
                |input: &str| match Instructions::try_from(input) {
                    Ok(instructions) => {
                        let mut cpu_state: CpuState = CpuState::default();

                        let sum_of_first_six_signal_strengths_where_cycle_mod_40_is_20: i32 =
                            cpu_state
                                .execute(instructions.iter())
                                .filter(CpuState::cycle_mod_40_is_20)
                                .map(CpuState::signal_strength)
                                .take(6_usize)
                                .sum();

                        println!(
                            "sum_of_first_six_signal_strengths_where_cycle_mod_40_is_20 == \
                            {sum_of_first_six_signal_strengths_where_cycle_mod_40_is_20}"
                        );
                    }
                    Err(error) => {
                        panic!("{error:#?}")
                    }
                },
            )
        }
    {
        eprintln!(
            "Encountered error {} when opening file \"{}\"",
            err, input_file_path
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INSTRUCTIONS_STR_1: &str = "\
        noop\n\
        addx 3\n\
        addx -5";
    const INSTRUCTIONS_STR_2: &str = "\
        addx 15\n\
        addx -11\n\
        addx 6\n\
        addx -3\n\
        addx 5\n\
        addx -1\n\
        addx -8\n\
        addx 13\n\
        addx 4\n\
        noop\n\
        addx -1\n\
        addx 5\n\
        addx -1\n\
        addx 5\n\
        addx -1\n\
        addx 5\n\
        addx -1\n\
        addx 5\n\
        addx -1\n\
        addx -35\n\
        addx 1\n\
        addx 24\n\
        addx -19\n\
        addx 1\n\
        addx 16\n\
        addx -11\n\
        noop\n\
        noop\n\
        addx 21\n\
        addx -15\n\
        noop\n\
        noop\n\
        addx -3\n\
        addx 9\n\
        addx 1\n\
        addx -3\n\
        addx 8\n\
        addx 1\n\
        addx 5\n\
        noop\n\
        noop\n\
        noop\n\
        noop\n\
        noop\n\
        addx -36\n\
        noop\n\
        addx 1\n\
        addx 7\n\
        noop\n\
        noop\n\
        noop\n\
        addx 2\n\
        addx 6\n\
        noop\n\
        noop\n\
        noop\n\
        noop\n\
        noop\n\
        addx 1\n\
        noop\n\
        noop\n\
        addx 7\n\
        addx 1\n\
        noop\n\
        addx -13\n\
        addx 13\n\
        addx 7\n\
        noop\n\
        addx 1\n\
        addx -33\n\
        noop\n\
        noop\n\
        noop\n\
        addx 2\n\
        noop\n\
        noop\n\
        noop\n\
        addx 8\n\
        noop\n\
        addx -1\n\
        addx 2\n\
        addx 1\n\
        noop\n\
        addx 17\n\
        addx -9\n\
        addx 1\n\
        addx 1\n\
        addx -3\n\
        addx 11\n\
        noop\n\
        noop\n\
        addx 1\n\
        noop\n\
        addx 1\n\
        noop\n\
        noop\n\
        addx -13\n\
        addx -19\n\
        addx 1\n\
        addx 3\n\
        addx 26\n\
        addx -30\n\
        addx 12\n\
        addx -1\n\
        addx 3\n\
        addx 1\n\
        noop\n\
        noop\n\
        noop\n\
        addx -9\n\
        addx 18\n\
        addx 1\n\
        addx 2\n\
        noop\n\
        noop\n\
        addx 9\n\
        noop\n\
        noop\n\
        noop\n\
        addx -1\n\
        addx 2\n\
        addx -37\n\
        addx 1\n\
        addx 3\n\
        noop\n\
        addx 15\n\
        addx -21\n\
        addx 22\n\
        addx -6\n\
        addx 1\n\
        noop\n\
        addx 2\n\
        addx 1\n\
        noop\n\
        addx -10\n\
        noop\n\
        noop\n\
        addx 20\n\
        addx 1\n\
        addx 2\n\
        addx 2\n\
        addx -6\n\
        addx -11\n\
        noop\n\
        noop\n\
        noop";

    #[test]
    fn test_instructions_try_from_str() {
        assert_eq!(
            Instructions::try_from(INSTRUCTIONS_STR_1),
            Ok(example_1_instructions())
        );
        assert_eq!(
            Instructions::try_from(INSTRUCTIONS_STR_2),
            Ok(example_2_instructions())
        );
    }

    #[test]
    fn test_execute() {
        let mut cpu_state: CpuState = CpuState::default();

        macro_rules! cpu_states {
            [$($cycle:expr => $x:expr,)*] => { vec![ $( CpuState { cycle: $cycle, x: $x }, )* ] };
        }

        assert_eq!(
            cpu_state
                .execute(example_1_instructions().iter())
                .collect::<Vec<CpuState>>(),
            cpu_states![
                1 => 1,
                2 => 1,
                3 => 1,
                4 => 4,
                5 => 4,
            ]
        );

        assert_eq!(
            cpu_state,
            CpuState {
                cycle: 6_u32,
                x: -1_i32
            }
        );
    }

    #[test]
    fn test_signal_strengths() {
        let mut cpu_state: CpuState = CpuState::default();

        let signal_strengths_where_cycle_mod_40_is_20: Vec<i32> = cpu_state
            .execute(example_2_instructions().iter())
            .filter(CpuState::cycle_mod_40_is_20)
            .map(CpuState::signal_strength)
            .collect();

        assert_eq!(
            signal_strengths_where_cycle_mod_40_is_20,
            vec![420, 1140, 1800, 2940, 2880, 3960]
        );
        assert_eq!(
            signal_strengths_where_cycle_mod_40_is_20
                .iter()
                .sum::<i32>(),
            13140
        );
    }

    fn example_1_instructions() -> Instructions {
        use Instruction::*;

        Instructions(vec![Noop, AddX(3), AddX(-5)])
    }

    fn example_2_instructions() -> Instructions {
        use Instruction::*;

        Instructions(vec![
            AddX(15),
            AddX(-11),
            AddX(6),
            AddX(-3),
            AddX(5),
            AddX(-1),
            AddX(-8),
            AddX(13),
            AddX(4),
            Noop,
            AddX(-1),
            AddX(5),
            AddX(-1),
            AddX(5),
            AddX(-1),
            AddX(5),
            AddX(-1),
            AddX(5),
            AddX(-1),
            AddX(-35),
            AddX(1),
            AddX(24),
            AddX(-19),
            AddX(1),
            AddX(16),
            AddX(-11),
            Noop,
            Noop,
            AddX(21),
            AddX(-15),
            Noop,
            Noop,
            AddX(-3),
            AddX(9),
            AddX(1),
            AddX(-3),
            AddX(8),
            AddX(1),
            AddX(5),
            Noop,
            Noop,
            Noop,
            Noop,
            Noop,
            AddX(-36),
            Noop,
            AddX(1),
            AddX(7),
            Noop,
            Noop,
            Noop,
            AddX(2),
            AddX(6),
            Noop,
            Noop,
            Noop,
            Noop,
            Noop,
            AddX(1),
            Noop,
            Noop,
            AddX(7),
            AddX(1),
            Noop,
            AddX(-13),
            AddX(13),
            AddX(7),
            Noop,
            AddX(1),
            AddX(-33),
            Noop,
            Noop,
            Noop,
            AddX(2),
            Noop,
            Noop,
            Noop,
            AddX(8),
            Noop,
            AddX(-1),
            AddX(2),
            AddX(1),
            Noop,
            AddX(17),
            AddX(-9),
            AddX(1),
            AddX(1),
            AddX(-3),
            AddX(11),
            Noop,
            Noop,
            AddX(1),
            Noop,
            AddX(1),
            Noop,
            Noop,
            AddX(-13),
            AddX(-19),
            AddX(1),
            AddX(3),
            AddX(26),
            AddX(-30),
            AddX(12),
            AddX(-1),
            AddX(3),
            AddX(1),
            Noop,
            Noop,
            Noop,
            AddX(-9),
            AddX(18),
            AddX(1),
            AddX(2),
            Noop,
            Noop,
            AddX(9),
            Noop,
            Noop,
            Noop,
            AddX(-1),
            AddX(2),
            AddX(-37),
            AddX(1),
            AddX(3),
            Noop,
            AddX(15),
            AddX(-21),
            AddX(22),
            AddX(-6),
            AddX(1),
            Noop,
            AddX(2),
            AddX(1),
            Noop,
            AddX(-10),
            Noop,
            Noop,
            AddX(20),
            AddX(1),
            AddX(2),
            AddX(2),
            AddX(-6),
            AddX(-11),
            Noop,
            Noop,
            Noop,
        ])
    }
}
