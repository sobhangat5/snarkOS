// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkOS library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{CurrentAleo, CurrentNetwork, Developer};

use snarkvm::{
    console::program::ProgramOwner,
    prelude::{
        block::Transaction,
        deployment_cost,
        query::Query,
        store::{helpers::memory::ConsensusMemory, ConsensusStore},
        PrivateKey,
        ProgramID,
        VM,
    },
};

use anyhow::{anyhow, bail, Result};
use clap::Parser;
use colored::Colorize;
use std::str::FromStr;

/// Deploys an Aleo program.
#[derive(Debug, Parser)]
pub struct Deploy {
    /// The name of the program to deploy.
    program_id: ProgramID<CurrentNetwork>,
    /// A path to a directory containing a manifest file. Defaults to the current working directory.
    #[clap(long)]
    path: Option<String>,
    /// The private key used to generate the deployment.
    #[clap(short, long)]
    private_key: String,
    /// The endpoint to query node state from.
    #[clap(short, long)]
    query: String,
    /// The priority fee in microcredits.
    #[clap(short, long)]
    fee: u64,
    /// The record to spend the fee from.
    #[clap(short, long)]
    record: String,
    /// The endpoint used to broadcast the generated transaction.
    #[clap(short, long, conflicts_with = "dry_run")]
    broadcast: Option<String>,
    /// Performs a dry-run of transaction generation.
    #[clap(short, long, conflicts_with = "broadcast")]
    dry_run: bool,
    /// Store generated deployment transaction to a local file.
    #[clap(long)]
    store: Option<String>,
}

impl Deploy {
    /// Deploys an Aleo program.
    pub fn parse(self) -> Result<String> {
        // Ensure that the user has specified an action.
        if !self.dry_run && self.broadcast.is_none() && self.store.is_none() {
            bail!("❌ Please specify one of the following actions: --broadcast, --dry-run, --store");
        }

        // Specify the query
        let query = Query::from(&self.query);

        // Retrieve the private key.
        let private_key = PrivateKey::from_str(&self.private_key)?;

        // Fetch the package from the directory.
        let package = Developer::parse_package(self.program_id, self.path)?;

        println!("📦 Creating deployment transaction for '{}'...\n", &self.program_id.to_string().bold());

        // Generate the deployment
        let deployment = package.deploy::<CurrentAleo>(None)?;
        let deployment_id = deployment.to_deployment_id()?;

        // Generate the deployment transaction.
        let transaction = {
            // Initialize an RNG.
            let rng = &mut rand::thread_rng();

            // Initialize the VM.
            let store = ConsensusStore::<CurrentNetwork, ConsensusMemory<CurrentNetwork>>::open(None)?;
            let vm = VM::from(store)?;

            // Compute the minimum deployment cost.
            let (minimum_deployment_cost, (_, _)) = deployment_cost(&deployment)?;
            // Determine the fee.
            let fee_in_microcredits = minimum_deployment_cost
                .checked_add(self.fee)
                .ok_or_else(|| anyhow!("Fee overflowed for a deployment transaction"))?;

            // Prepare the fees.
            let fee_record = Developer::parse_record(&private_key, &self.record)?;
            let (_, fee) =
                vm.execute_fee_raw(&private_key, fee_record, fee_in_microcredits, deployment_id, Some(query), rng)?;

            // Construct the owner.
            let owner = ProgramOwner::new(&private_key, deployment_id, rng)?;

            // Create a new transaction.
            Transaction::from_deployment(owner, deployment, fee)?
        };
        println!("✅ Created deployment transaction for '{}'", self.program_id.to_string().bold());

        // Determine if the transaction should be broadcast, stored, or displayed to user.
        Developer::handle_transaction(
            self.broadcast,
            self.dry_run,
            self.store,
            transaction,
            self.program_id.to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{Command, CLI};

    #[test]
    fn clap_snarkos_deploy() {
        let arg_vec = vec![
            "snarkos",
            "developer",
            "deploy",
            "--private-key",
            "PRIVATE_KEY",
            "--query",
            "QUERY",
            "--fee",
            "77",
            "--record",
            "RECORD",
            "hello.aleo",
        ];
        let cli = CLI::parse_from(arg_vec);

        if let Command::Developer(Developer::Deploy(deploy)) = cli.command {
            assert_eq!(deploy.program_id, "hello.aleo".try_into().unwrap());
            assert_eq!(deploy.private_key, "PRIVATE_KEY");
            assert_eq!(deploy.query, "QUERY");
            assert_eq!(deploy.fee, 77);
            assert_eq!(deploy.record, "RECORD");
        } else {
            panic!("Unexpected result of clap parsing!");
        }
    }
}
