// Copyright (C) 2019-2021 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use crate::{prelude::*, Network};
use snarkvm_utilities::{FromBytes, ToBytes};

use anyhow::Result;
use std::io::{Read, Result as IoResult, Write};

/// The transaction kernel contains core (public) transaction components,
/// A signed transaction kernel implies the caller has authorized the execution
/// of the transaction, and implies these values are admissible by the ledger.
#[derive(Derivative)]
#[derivative(
    Clone(bound = "N: Network"),
    Debug(bound = "N: Network"),
    PartialEq(bound = "N: Network"),
    Eq(bound = "N: Network")
)]
pub struct TransactionKernel<N: Network> {
    /// The network ID.
    network_id: u16,
    /// The serial numbers of the input records.
    serial_numbers: Vec<N::SerialNumber>,
    /// The commitments of the output records.
    commitments: Vec<N::Commitment>,
    /// A value balance is the difference between the input and output record values.
    /// The value balance serves as the transaction fee for the miner. Only coinbase transactions
    /// may possess a negative value balance representing tokens being minted.
    value_balance: AleoAmount,
    /// Publicly-visible data associated with the transaction.
    memo: Memo<N>,
}

impl<N: Network> TransactionKernel<N> {
    /// Initializes a new instance of a transaction kernel.
    #[inline]
    pub fn new(
        serial_numbers: Vec<N::SerialNumber>,
        commitments: Vec<N::Commitment>,
        value_balance: AleoAmount,
        memo: Memo<N>,
    ) -> Result<Self> {
        // Construct the transaction kernel.
        let kernel = Self {
            network_id: N::NETWORK_ID,
            serial_numbers,
            commitments,
            value_balance,
            memo,
        };

        // Ensure the transaction kernel is well-formed.
        match kernel.is_valid() {
            true => Ok(kernel),
            false => Err(
                DPCError::InvalidKernel(N::NETWORK_ID, kernel.serial_numbers.len(), kernel.commitments.len()).into(),
            ),
        }
    }

    /// Returns `true` if the transaction kernel is well-formed.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.network_id == N::NETWORK_ID
            && self.serial_numbers.len() == N::NUM_INPUT_RECORDS
            && self.commitments.len() == N::NUM_OUTPUT_RECORDS
    }

    /// Returns the network ID.
    #[inline]
    pub fn network_id(&self) -> u16 {
        self.network_id
    }

    /// Returns a reference to the serial numbers.
    #[inline]
    pub fn serial_numbers(&self) -> &Vec<N::SerialNumber> {
        &self.serial_numbers
    }

    /// Returns a reference to the commitments.
    #[inline]
    pub fn commitments(&self) -> &Vec<N::Commitment> {
        &self.commitments
    }

    /// Returns a reference to the value balance.
    #[inline]
    pub fn value_balance(&self) -> &AleoAmount {
        &self.value_balance
    }

    /// Returns a reference to the memo.
    pub fn memo(&self) -> &Memo<N> {
        &self.memo
    }

    #[inline]
    pub fn to_signature_message(&self) -> Result<Vec<u8>> {
        match self.is_valid() {
            true => self.to_bytes_le(),
            false => {
                Err(DPCError::InvalidKernel(self.network_id, self.serial_numbers.len(), self.commitments.len()).into())
            }
        }
    }
}

impl<N: Network> ToBytes for TransactionKernel<N> {
    #[inline]
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        // Ensure the correct number of serial numbers and commitments are provided.
        if !self.is_valid() {
            return Err(
                DPCError::InvalidKernel(self.network_id, self.serial_numbers.len(), self.commitments.len()).into(),
            );
        }

        self.network_id.write_le(&mut writer)?;
        self.serial_numbers.write_le(&mut writer)?;
        self.commitments.write_le(&mut writer)?;
        self.value_balance.write_le(&mut writer)?;
        self.memo.write_le(&mut writer)
    }
}

impl<N: Network> FromBytes for TransactionKernel<N> {
    #[inline]
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let network_id: u16 = FromBytes::read_le(&mut reader)?;

        // Ensure the correct network ID is read in.
        if network_id != N::NETWORK_ID {
            return Err(DPCError::InvalidKernel(network_id, N::NUM_INPUT_RECORDS, N::NUM_OUTPUT_RECORDS).into());
        }

        let mut serial_numbers = Vec::<N::SerialNumber>::with_capacity(N::NUM_INPUT_RECORDS);
        for _ in 0..N::NUM_INPUT_RECORDS {
            serial_numbers.push(FromBytes::read_le(&mut reader)?);
        }

        let mut commitments = Vec::<N::Commitment>::with_capacity(N::NUM_OUTPUT_RECORDS);
        for _ in 0..N::NUM_OUTPUT_RECORDS {
            commitments.push(FromBytes::read_le(&mut reader)?);
        }

        let value_balance: AleoAmount = FromBytes::read_le(&mut reader)?;
        let memo: Memo<N> = FromBytes::read_le(&mut reader)?;

        Ok(Self::new(serial_numbers, commitments, value_balance, memo)
            .expect("Failed to initialize a transaction kernel"))
    }
}
