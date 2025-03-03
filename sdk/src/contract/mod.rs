use std::prelude::v1::*;
pub mod builtins;
pub mod schema;
use self::generator::CellMetaTransaction;
use self::schema::*;

use crate::ckb_types::packed::{CellInput, CellOutput, CellOutputBuilder, Uint64};
use crate::ckb_types::{bytes::Bytes, packed, prelude::*};

#[cfg(not(feature = "script"))]
pub mod generator;
#[cfg(not(feature = "script"))]
use self::generator::{CellQuery, GeneratorMiddleware};
#[cfg(not(feature = "script"))]
use crate::chain::CellOutputWithData;
#[cfg(not(feature = "script"))]
use crate::ckb_types::core::TransactionView;
#[cfg(not(feature = "script"))]
use crate::ckb_types::{core::TransactionBuilder, H256};
#[cfg(not(feature = "script"))]
use ckb_hash::blake2b_256;
#[cfg(not(feature = "script"))]
use ckb_jsonrpc_types::{CellDep, DepType, JsonBytes, OutPoint, Script};
use ckb_types::core::cell::CellMeta;

#[cfg(not(feature = "script"))]
use std::fs;
#[cfg(not(feature = "script"))]
use std::path::PathBuf;

#[cfg(not(feature = "script"))]
use std::sync::{Arc, Mutex};

#[cfg(not(feature = "script"))]
#[derive(Debug, Clone)]
pub enum ContractSource {
    LocalPath(PathBuf),
    Immediate(Bytes),
    Chain(OutPoint),
}
#[cfg(not(feature = "script"))]
impl ContractSource {
    pub fn load_from_path(path: PathBuf) -> std::io::Result<Bytes> {
        let file = fs::read(path)?;
        println!("SUDT CODE SIZE: {}", file.len());
        Ok(Bytes::from(file))
    }
}

#[cfg(not(feature = "script"))]
#[derive(Clone, PartialEq)]
pub enum ContractField {
    Args,
    Data,
    LockScript,
    TypeScript,
    Capacity,
}

#[cfg(not(feature = "script"))]
#[derive(Clone, PartialEq)]
pub enum TransactionField {
    ResolvedInputs,
    Inputs,
    Outputs,
    Dependencies,
}

#[cfg(not(feature = "script"))]
#[derive(PartialEq)]
pub enum RuleScope {
    ContractField(ContractField),
    TransactionField(TransactionField),
}
#[cfg(not(feature = "script"))]
impl From<ContractField> for RuleScope {
    fn from(f: ContractField) -> Self {
        Self::ContractField(f)
    }
}
#[cfg(not(feature = "script"))]
impl From<TransactionField> for RuleScope {
    fn from(f: TransactionField) -> Self {
        Self::TransactionField(f)
    }
}
#[cfg(not(feature = "script"))]
#[derive(Clone)]
pub struct RuleContext {
    inner: CellMetaTransaction,
    pub idx: usize,
    pub curr_field: TransactionField,
}
#[cfg(not(feature = "script"))]
impl RuleContext {
    pub fn new(tx: impl Into<CellMetaTransaction>) -> Self {
        Self {
            inner: tx.into(),
            idx: 0,
            curr_field: TransactionField::Outputs,
        }
    }
    pub fn tx(mut self, tx: impl Into<CellMetaTransaction>) -> Self {
        self.inner = tx.into();
        self
    }

    pub fn get_tx(&self) -> CellMetaTransaction {
        self.inner.clone()
    }
    pub fn idx(&mut self, idx: usize) {
        self.idx = idx;
    }

    pub fn curr_field(&mut self, field: TransactionField) {
        self.curr_field = field;
    }

    pub fn load<A, D>(&self, scope: impl Into<RuleScope>) -> ContractCellField<A, D>
    where
        D: JsonByteConversion + MolConversion + BytesConversion + Clone + Default,
        A: JsonByteConversion + MolConversion + BytesConversion + Clone,
    {
        match scope.into() {
            RuleScope::ContractField(field) => match field {
                ContractField::Args => todo!(),
                ContractField::Data => match self.curr_field {
                    TransactionField::Outputs => {
                        let data_reader = self.inner.outputs_data();
                        let data_reader = data_reader.as_reader();
                        let data = data_reader.get(self.idx);
                        if let Some(data) = data {
                            ContractCellField::Data(D::from_bytes(data.raw_data().to_vec().into()))
                        } else {
                            ContractCellField::Data(D::default())
                        }
                    }
                    _ => ContractCellField::Data(D::default()),
                },
                ContractField::LockScript => todo!(),
                ContractField::TypeScript => todo!(),
                ContractField::Capacity => todo!(),
            },
            RuleScope::TransactionField(field) => match field {
                TransactionField::Inputs => ContractCellField::Inputs(
                    self.inner.inputs().into_iter().collect::<Vec<CellInput>>(),
                ),
                TransactionField::Outputs => ContractCellField::Outputs(
                    self.inner
                        .outputs_with_data_iter()
                        .collect::<Vec<CellOutputWithData>>(),
                ),
                TransactionField::Dependencies => ContractCellField::CellDeps(
                    self.inner
                        .cell_deps_iter()
                        .collect::<Vec<crate::ckb_types::packed::CellDep>>(),
                ),
                TransactionField::ResolvedInputs => {
                    ContractCellField::ResolvedInputs(self.inner.inputs.clone())
                }
            },
        }
    }
}

#[cfg(not(feature = "script"))]
pub struct OutputRule<A, D> {
    pub scope: RuleScope,
    pub rule: Box<dyn Fn(RuleContext) -> ContractCellField<A, D>>,
}

#[cfg(not(feature = "script"))]
impl<A, D> OutputRule<A, D> {
    pub fn new<F>(scope: impl Into<RuleScope>, rule: F) -> Self
    where
        F: 'static + Fn(RuleContext) -> ContractCellField<A, D>,
    {
        OutputRule {
            scope: scope.into(),
            rule: Box::new(rule),
        }
    }
    pub fn exec(&self, ctx: &RuleContext) -> ContractCellField<A, D> {
        self.rule.as_ref()(ctx.clone()) //call((ctx,))
    }
}

#[cfg(not(feature = "script"))]

pub enum ContractCellField<A, D> {
    Args(A),
    Data(D),
    LockScript(ckb_types::packed::Script),
    TypeScript(ckb_types::packed::Script),
    Capacity(Uint64),
    Inputs(Vec<CellInput>),
    ResolvedInputs(Vec<CellMeta>),
    Outputs(Vec<CellOutputWithData>),
    CellDeps(Vec<ckb_types::packed::CellDep>),
}
#[cfg(not(feature = "script"))]
#[derive(Default)]
#[cfg(not(feature = "script"))]
pub struct Contract<A, D> {
    pub source: Option<ContractSource>,
    pub data: D,
    pub args: A,
    pub lock: Option<Script>,
    pub type_: Option<Script>,
    pub code: Option<JsonBytes>,
    #[allow(clippy::type_complexity)]
    pub output_rules: Vec<OutputRule<A, D>>,
    pub input_rules: Vec<Box<dyn Fn(TransactionView) -> CellQuery>>,
}

#[cfg(not(feature = "script"))]
impl<A, D> Contract<A, D>
where
    D: JsonByteConversion + MolConversion + BytesConversion + Clone,
    A: JsonByteConversion + MolConversion + BytesConversion + Clone,
{
    // The lock script of the cell containing contract code
    pub fn lock(mut self, lock: Script) -> Self {
        self.lock = Some(lock);
        self
    }

    // The type script of the cell containing contract code
    pub fn type_(mut self, type_: Script) -> Self {
        self.type_ = Some(type_);
        self
    }

    pub fn data_hash(&self) -> Option<H256> {
        if let Some(data) = &self.code {
            let byte_slice = data.as_bytes();

            let raw_hash = blake2b_256(&byte_slice);
            H256::from_slice(&raw_hash).ok()
        } else {
            None
        }
    }

    // Returns a script structure which can be used as a lock or type script on other cells.
    // This is an easy way to let other cells use this contract
    pub fn as_script(&self) -> Option<ckb_jsonrpc_types::Script> {
        self.data_hash().map(|data_hash| {
            Script::from(
                packed::ScriptBuilder::default()
                    .args(self.args.to_bytes().pack())
                    .code_hash(data_hash.pack())
                    .hash_type(ckb_types::core::ScriptHashType::Data1.into())
                    .build(),
            )
        })
    }

    // Return a CellOutputWithData which is the code cell storing this contract's logic
    pub fn as_code_cell(&self) -> CellOutputWithData {
        let data: Bytes = self.code.clone().unwrap_or_default().into_bytes();
        let type_script = self.type_.clone().unwrap_or_default();
        let type_script = {
            if self.type_.is_some() {
                Some(ckb_types::packed::Script::from(type_script))
            } else {
                None
            }
        };

        let cell_output = CellOutputBuilder::default()
            .capacity((data.len() as u64).pack())
            .lock(self.lock.clone().unwrap_or_default().into())
            .type_(type_script.pack())
            .build();
        (cell_output, data)
    }

    pub fn script_hash(&self) -> Option<ckb_jsonrpc_types::Byte32> {
        let script: ckb_types::packed::Script = self.as_script().unwrap().into();
        Some(script.calc_script_hash().into())
    }

    pub fn as_cell_dep(&self, out_point: OutPoint) -> CellDep {
        CellDep {
            out_point,
            dep_type: DepType::Code,
        }
    }

    // Set data of a cell that will *reference* (i.e., use) this contract
    pub fn set_raw_data(&mut self, data: impl Into<JsonBytes>) {
        self.data = D::from_json_bytes(data.into());
    }

    pub fn set_data(&mut self, data: D) {
        self.data = data;
    }

    // Set args of a cell that will *reference* (i.e., use) this contract
    pub fn set_raw_args(&mut self, args: impl Into<JsonBytes>) {
        self.args = A::from_json_bytes(args.into());
    }

    pub fn set_args(&mut self, args: A) {
        self.args = args;
    }

    pub fn read_data(&self) -> D {
        self.data.clone()
    }

    pub fn read_args(&self) -> A {
        self.args.clone()
    }

    pub fn read_raw_data(&self, data: Bytes) -> D {
        D::from_bytes(data)
    }

    pub fn read_raw_args(&self, args: Bytes) -> A {
        A::from_bytes(args)
    }

    pub fn add_output_rule<F>(&mut self, scope: impl Into<RuleScope>, transform_func: F)
    where
        F: Fn(RuleContext) -> ContractCellField<A, D> + 'static,
    {
        self.output_rules
            .push(OutputRule::new(scope.into(), transform_func));
    }

    pub fn add_input_rule<F>(&mut self, query_func: F)
    where
        F: Fn(TransactionView) -> CellQuery + 'static,
    {
        self.input_rules.push(Box::new(query_func))
    }

    pub fn tx_template(&self) -> TransactionView {
        let arg_size = self.args.to_mol().as_builder().expected_length() as u64;
        let data_size = self.data.to_mol().as_builder().expected_length() as u64;
        println!("DATA SIZE EXPECTED: {:?}", data_size);
        let mut data = Vec::with_capacity(data_size as usize);
        (0..data_size as usize).into_iter().for_each(|_| {
            data.push(0u8);
        });
        let mut tx = TransactionBuilder::default()
            .output(
                CellOutput::new_builder()
                    .capacity((data_size + arg_size).pack())
                    .type_(Some(ckb_types::packed::Script::from(self.as_script().unwrap())).pack())
                    .build(),
            )
            .output_data(data.pack());

        if let Some(ContractSource::Chain(outp)) = self.source.clone() {
            tx = tx.cell_dep(self.as_cell_dep(outp).into());
        }

        tx.build()
    }
}
#[cfg(not(feature = "script"))]
impl<A, D> GeneratorMiddleware for Contract<A, D>
where
    D: JsonByteConversion + MolConversion + BytesConversion + Clone,
    A: JsonByteConversion + MolConversion + BytesConversion + Clone,
{
    fn update_query_register(
        &self,
        tx: CellMetaTransaction,
        query_register: Arc<Mutex<Vec<CellQuery>>>,
    ) {
        let queries = self.input_rules.iter().map(|rule| rule(tx.clone().tx));

        query_register.lock().unwrap().extend(queries);
    }
    fn pipe(
        &self,
        tx_meta: CellMetaTransaction,
        _query_queue: Arc<Mutex<Vec<CellQuery>>>,
    ) -> CellMetaTransaction {
        type OutputWithData = (CellOutput, Bytes);

        let tx = tx_meta.tx.clone();
        let tx_template = self.tx_template();

        let total_deps = tx
            .cell_deps()
            .as_builder()
            .extend(tx_template.cell_deps_iter())
            .build();
        let total_outputs = tx
            .outputs()
            .as_builder()
            .extend(tx_template.outputs())
            .build();
        let total_inputs = tx
            .inputs()
            .as_builder()
            .extend(tx_template.inputs())
            .build();
        let total_outputs_data = tx
            .outputs_data()
            .as_builder()
            .extend(tx_template.outputs_data())
            .build();
        let tx = tx
            .as_advanced_builder()
            .set_cell_deps(
                total_deps
                    .into_iter()
                    .collect::<Vec<crate::ckb_types::packed::CellDep>>(),
            )
            .set_outputs(
                total_outputs
                    .into_iter()
                    .collect::<Vec<crate::ckb_types::packed::CellOutput>>(),
            )
            .set_inputs(
                total_inputs
                    .into_iter()
                    .collect::<Vec<crate::ckb_types::packed::CellInput>>(),
            )
            .set_outputs_data(
                total_outputs_data
                    .into_iter()
                    .collect::<Vec<crate::ckb_types::packed::Bytes>>(),
            )
            .build();
        let mut idx = 0;
        let outputs = tx.clone().outputs().into_iter().filter_map(|output| {
            let self_script_hash: ckb_types::packed::Byte32 = self.script_hash().unwrap().into();

            if let Some(type_) = output.type_().to_opt() {
                if type_.calc_script_hash() == self_script_hash {
                    return Some((idx, tx.output_with_data(idx).unwrap()));
                }
            }

            if output.lock().calc_script_hash() == self_script_hash {
                return Some((idx, tx.output_with_data(idx).unwrap()));
            }

            idx += 1;
            None
        });

        let mut ctx = RuleContext::new(tx_meta.clone());

        let outputs = outputs
            .into_iter()
            .map(|output_with_idx| {
                ctx.idx(output_with_idx.0);
                let processed = self.output_rules.iter().fold(output_with_idx.1, |output, rule| {
                    let data = self.read_raw_data(output.1.clone());
                    println!("Data before update {:?}", data.to_mol());
                    let updated_field = rule.exec(&ctx);
                    match updated_field {
                        ContractCellField::Args(_) => todo!(),
                        ContractCellField::Data(d) => {
                            if rule.scope != ContractField::Data.into() {
                                panic!("Error, mismatch of output rule scope and returned field");
                            }
                            let updated_tx = ctx.get_tx();
                            let inner_tx_view = updated_tx.tx.clone();
                            let updated_outputs_data = inner_tx_view.outputs_with_data_iter()
                                .enumerate().map(|(i, output)| {
                                    if i == ctx.idx {
                                       (output.0, d.to_bytes())
                                    } else {
                                        output
                                    }
                                }).collect::<Vec<CellOutputWithData>>();
                            let updated_inner_tx = inner_tx_view.as_advanced_builder()
                                .set_outputs(updated_outputs_data.iter().map(|o| o.0.clone()).collect::<Vec<_>>())
                                .set_outputs_data(updated_outputs_data.iter().map(|o| o.1.pack()).collect::<Vec<_>>())
                                .build();
                            let updated_tx = updated_tx.tx(updated_inner_tx);
                            ctx = ctx.clone().tx(updated_tx);
                            (output.0, d.to_bytes())
                        },
                        ContractCellField::LockScript(_) => todo!(),
                        ContractCellField::TypeScript(_) => todo!(),
                        ContractCellField::Capacity(_) => todo!(),
                        _ => {
                            panic!("Error: Contract-level rule attempted transaction-level update.")
                        }
                    }
                });
                println!("Output bytes of processed output: {:?}", processed.1.pack());
                processed
            })
            .collect::<Vec<OutputWithData>>();

        let final_inner_tx = tx
            .as_advanced_builder()
            .set_outputs(
                outputs
                    .iter()
                    .map(|out| out.0.clone())
                    .collect::<Vec<CellOutput>>(),
            )
            .set_outputs_data(
                outputs
                    .iter()
                    .map(|out| out.1.clone().pack())
                    .collect::<Vec<ckb_types::packed::Bytes>>(),
            )
            .build();
        tx_meta.tx(final_inner_tx)
    }
}
