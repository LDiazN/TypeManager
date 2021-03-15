/*
    Main driver code
*/

// Rust imports
use std::io;
use std::io::Write;

// Internal imports
use crate::type_system::*;


/// Our program object
pub struct Program {
    running: bool,
    manager: TypeManager
}

/// possible errors
pub enum ProgramError {
    NotEnoughArgs,
    TooManyArgs,
    InvalidAction(String),
    InvalidArgument(String)
}

/// Possible actions
pub enum Action {
    Display(Name),
    AddStruct(Name, TypeList),      // name, members
    AddUnion(Name, TypeList),       // name, variants
    AddAtomic(Name, usize, usize),  // name, representation, alignment
    Exit
}

impl Program {

    // Crea un programa nuevo listo para correr
    pub fn new() -> Program {
        Program {
            manager: TypeManager::new(),
            running: true
        }
    }

    /// Tells if this program should run
    pub fn should_run(&self) -> bool {
        self.running
    }

    /// Run an iteration for the program
    pub fn run(&mut self) {

        // Command buffer: store user input in this line
        let mut line = String::new();

        print!(">> "); // print prompt
        // flush so the print! doesn't mess up the execution order with read_line
        io::stdout().flush().expect("Couldn't flush stdout"); 

        // Read a single line
        if let Err(_) = io::stdin().read_line(&mut line) { panic!("Error leyendo input D:") }
        
        // Parse next action
        let next_action = match Program::parse(line) {
            Err(e) => { println!("[ERROR]: {}", e.display()); return },
            Ok(a)  => a
        };

        // Function to handle type errors 
        let handle_error = |e : TypeError| {
            println!("[TYPE ERROR]: {}", e.display());
            let res : Option<TypeError> = None;

            res
        };

        // Create a new type as described and handle error if necessary
        match next_action {
            Action::Exit => self.running = false,
            Action::Display(s) => { 
                    self.manager
                    .display(&s)
                    .and_then(|msg|
                        { println!("Símbolo: {}\n{}", s, msg); Ok(()) }
                    )
                    .err()
                    .and_then(handle_error); 
                }
            Action::AddAtomic(name, repr, align) => {
                    self.manager
                    .add(
                        name,
                        Type::Atomic( 
                            Atomic::new(repr, align)
                        )
                    )
                    .err()
                    .and_then(handle_error);
                },
            Action::AddStruct(name, members) => {
                self.manager
                .add(
                    name, 
                    Type::Struct(
                        Struct::new(members)
                    )
                )
                .err()
                .and_then(handle_error);
            },
            Action::AddUnion(name, variants) => {
                self.manager
                .add(
                    name, 
                    Type::Union(
                        Union::new(variants)
                    )
                )
                .err()
                .and_then(handle_error);
            }
        };
    }

    /// Get next action from user input
    fn parse(input: String) -> Result<Action, ProgramError>{
        let mut input = input.split_whitespace();

        // Try to Parse verb from input
        let action = match input.next() {
            None  => return Err( ProgramError::NotEnoughArgs ),
            Some(s) => s.to_lowercase()
        };

        match action.as_str() {
            "salir"     => Ok(Action::Exit),
            "union"     => Program::parse_action(input, Action::AddUnion),
            "struct"    => Program::parse_action(input, Action::AddStruct),
            "atomico"   => Program::parse_atomic(input),
            "describir" => Program::parse_display(input),
            _        => Err( ProgramError::InvalidAction(action) )
        }
    }

    /// Parse properties from token iterator and create a new action with the given constructor
    fn parse_action<'a, I>(input:  I, act : fn (Name, TypeList) -> Action) -> Result<Action, ProgramError> 
        where 
            I: Iterator<Item=&'a str>
    {
        let mut input = input; // honestly i don't get why this is necessary for this function to type check but ok

        // parse name, if not found, then not enough arguments error is raised
        let name = match &mut input.next() {
            None    => return Err( ProgramError::NotEnoughArgs ),
            Some(e) => e.to_string()
        };

        // Create a typelist from the rest of our tokens
        let types : TypeList= input.map(|s| s.to_string()).collect();

        Ok(act(name, types))
    }

    /// Parse atomic type
    fn parse_atomic<'a, I>(input: I) -> Result<Action, ProgramError> 
        where
            I: Iterator<Item = &'a str>
    {
        let mut input = input;

        // Try to parse name
        let name = match input.next() {
            Some(s) => s,
            None    => return Err( ProgramError::NotEnoughArgs )
        };

        // Try to get first argument
        let repr = match input.next() {
            Some(s) => s,
            None    => return Err(ProgramError::NotEnoughArgs)
        };

        // try to get second argument
        let align = match input.next() {
            Some(s) => s,
            None    => return Err(ProgramError::NotEnoughArgs)
        };

        // check if too many arguments
        if let Some(_) = input.next() {
            return Err(ProgramError::TooManyArgs)
        };

        // Try to parse arguments
        let repr = match repr.parse::<usize>() {
            Ok(n)  => n,
            Err(_) => return Err(ProgramError::InvalidArgument(repr.to_string()))
        };

        let align = match align.parse::<usize>() {
            Ok(n)  => n,
            Err(_) => return Err(ProgramError::InvalidArgument(repr.to_string()))
        };

        // return our new atomic type
        Ok(Action::AddAtomic(name.to_string(), repr, align))
    }

    /// Parse a display action
    fn parse_display<'a, I>(input: I) -> Result<Action, ProgramError> 
        where 
            I: Iterator<Item = &'a str>
    {
        let mut input = input;

        // Parse name
        let name = match input.next() {
            None    => return Err(ProgramError::NotEnoughArgs),
            Some(s) => s
        };

        // Check if too many arguments
        if let Some(_) = input.next() {
            return Err(ProgramError::TooManyArgs)
        };

        Ok(Action::Display(name.to_string()))
    }
}

impl ProgramError {

    /// Get human readable description for this error
    pub fn display(&self) -> String {
        match self {
            ProgramError::InvalidAction(s) => {
                format!("'{}' no es una acción válida", s)
            },
            ProgramError::InvalidArgument(s) => {
                format!("Este no es un argumento válido: {}", s)
            },
            ProgramError::NotEnoughArgs => {
                format!("No hay suficientes argumentos")
            },
            ProgramError::TooManyArgs => {
                format!("Demasiados argumentos")
            }
        }
    }
}