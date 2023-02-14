#[allow(non_camel_case_types)]
pub enum MachineRegister {
    x86_64(gcf_x86_64::Register),
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum AccessMode {
    // Bit
    BI,
    /// Quarter Integer
    QI,
    /// Half Integer
    HI,
    /// Single Integer
    SI,
    /// Partial Single Integer
    PSI,
    /// Double Integer
    DI,
}

pub struct Register {
    number: usize,
    machine_register: Option<MachineRegister>,
    natural_mode: AccessMode,
}

impl Register {
    pub fn vreg(number: usize, natural_mode: AccessMode) -> Self {
        Self {
            number,
            natural_mode,
            machine_register: None,
        }
    }

    pub fn preg(
        number: usize,
        machine_register: MachineRegister,
        natural_mode: AccessMode,
    ) -> Self {
        Self {
            number,
            machine_register: Some(machine_register),
            natural_mode,
        }
    }

    pub fn number(&self) -> usize {
        self.number
    }

    pub fn machine_register(&self) -> Option<&MachineRegister> {
        self.machine_register.as_ref()
    }

    pub fn x86_64_machine_register(&self) -> Option<gcf_x86_64::Register> {
        self.machine_register.as_ref().and_then(|mc| match mc {
            MachineRegister::x86_64(x86_64) => Some(*x86_64),
        })
    }

    pub fn natural_mode(&self) -> AccessMode {
        self.natural_mode
    }
}
