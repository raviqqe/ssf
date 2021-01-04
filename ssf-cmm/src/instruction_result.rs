pub struct InstructionResult {
    instructions: Vec<cmm::ir::Instruction>,
    variable: cmm::ir::Variable,
}

impl InstructionResult {
    pub fn new(instructions: Vec<cmm::ir::Instruction>, variable: impl Into<String>) -> Self {
        Self {
            instructions,
            variable,
        }
    }

    pub fn instructions(&self) -> &[cmm::ir::Instruction] {
        &self.instructions
    }

    pub fn variable(&self) -> &cmm::ir::Variable {
        &self.variable
    }
}
