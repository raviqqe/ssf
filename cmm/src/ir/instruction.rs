use super::address_calculation::AddressCalculation;
use super::allocate_heap::AllocateHeap;
use super::allocate_stack::AllocateStack;
use super::assignment::Assignment;
use super::atomic_load::AtomicLoad;
use super::atomic_store::AtomicStore;
use super::bitcast::Bitcast;
use super::call::Call;
use super::compare_and_swap::CompareAndSwap;
use super::deconstruct_record::DeconstructRecord;
use super::if_::If;
use super::load::Load;
use super::primitive_operation::PrimitiveOperation;
use super::return_::Return;
use super::store::Store;
use super::switch::Switch;

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    AddressCalculation(AddressCalculation),
    AllocateHeap(AllocateHeap),
    AllocateStack(AllocateStack),
    Assignment(Assignment),
    AtomicLoad(AtomicLoad),
    AtomicStore(AtomicStore),
    Bitcast(Bitcast),
    Call(Call),
    CompareAndSwap(CompareAndSwap),
    DeconstructRecord(DeconstructRecord),
    If(If),
    Load(Load),
    PrimitiveOperation(PrimitiveOperation),
    Return(Return),
    Store(Store),
    Switch(Switch),
    Unreachable,
}

impl From<AllocateHeap> for Instruction {
    fn from(allocate_heap: AllocateHeap) -> Self {
        Self::AllocateHeap(allocate_heap)
    }
}

impl From<AllocateStack> for Instruction {
    fn from(allocate: AllocateStack) -> Self {
        Self::AllocateStack(allocate)
    }
}

impl From<Assignment> for Instruction {
    fn from(assignment: Assignment) -> Self {
        Self::Assignment(assignment)
    }
}

impl From<AtomicStore> for Instruction {
    fn from(store: AtomicStore) -> Self {
        Self::AtomicStore(store)
    }
}

impl From<If> for Instruction {
    fn from(if_: If) -> Self {
        Self::If(if_)
    }
}

impl From<Return> for Instruction {
    fn from(return_: Return) -> Self {
        Self::Return(return_)
    }
}

impl From<Store> for Instruction {
    fn from(store: Store) -> Self {
        Self::Store(store)
    }
}

impl From<Switch> for Instruction {
    fn from(switch: Switch) -> Self {
        Self::Switch(switch)
    }
}

impl From<AddressCalculation> for Instruction {
    fn from(calculation: AddressCalculation) -> Self {
        Self::AddressCalculation(calculation)
    }
}

impl From<AtomicLoad> for Instruction {
    fn from(load: AtomicLoad) -> Self {
        Self::AtomicLoad(load)
    }
}

impl From<Bitcast> for Instruction {
    fn from(bitcast: Bitcast) -> Self {
        Self::Bitcast(bitcast)
    }
}

impl From<Call> for Instruction {
    fn from(call: Call) -> Self {
        Self::Call(call)
    }
}

impl From<CompareAndSwap> for Instruction {
    fn from(compare_and_swap: CompareAndSwap) -> Self {
        Self::CompareAndSwap(compare_and_swap)
    }
}

impl From<DeconstructRecord> for Instruction {
    fn from(deconstruct_record: DeconstructRecord) -> Self {
        Self::DeconstructRecord(deconstruct_record)
    }
}

impl From<Load> for Instruction {
    fn from(load: Load) -> Self {
        Self::Load(load)
    }
}

impl From<PrimitiveOperation> for Instruction {
    fn from(operation: PrimitiveOperation) -> Self {
        Self::PrimitiveOperation(operation)
    }
}
