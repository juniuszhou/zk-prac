// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";

/// @notice Public outputs committed by `program/src/main.rs` via `commit_slice`.
struct PublicValuesStruct {
    uint32 n;
    uint32 a;
    uint32 b;
}

/// @title Fibonacci SP1 verifier app
/// @notice Verifies an EVM-compatible SP1 proof that the Fibonacci zkVM program was executed
///         correctly, then returns the decoded public values (n, F(n-1), F(n)).
contract Fibonacci {
    /// @notice SP1 verifier (SP1VerifierGateway or version-specific SP1Verifier).
    /// @dev Deployed addresses: https://github.com/succinctlabs/sp1-contracts/tree/main/contracts/deployments
    address public immutable verifier;

    /// @notice Verification key for `fibonacci-program` (from `cargo run --bin vkey`).
    bytes32 public immutable fibonacciProgramVKey;

    event FibonacciVerified(uint32 n, uint32 a, uint32 b, address indexed caller);

    constructor(address _verifier, bytes32 _fibonacciProgramVKey) {
        verifier = _verifier;
        fibonacciProgramVKey = _fibonacciProgramVKey;
    }

    /// @notice Verify SP1 proof and decode public values.
    /// @param publicValues ABI-encoded `PublicValuesStruct` bytes committed in the zkVM.
    /// @param proofBytes Groth16 or PLONK proof bytes from `cargo run --bin evm`.
    /// @return n Input index passed to the program.
    /// @return a Fibonacci F(n-1).
    /// @return b Fibonacci F(n).
    function verifyFibonacciProof(bytes calldata publicValues, bytes calldata proofBytes)
        external
        view
        returns (uint32 n, uint32 a, uint32 b)
    {
        ISP1Verifier(verifier).verifyProof(fibonacciProgramVKey, publicValues, proofBytes);
        PublicValuesStruct memory pv = abi.decode(publicValues, (PublicValuesStruct));
        return (pv.n, pv.a, pv.b);
    }

    /// @notice Same as `verifyFibonacciProof` but emits an event (for indexers).
    function verifyFibonacciProofAndEmit(bytes calldata publicValues, bytes calldata proofBytes)
        external
        returns (uint32 n, uint32 a, uint32 b)
    {
        (n, a, b) = this.verifyFibonacciProof(publicValues, proofBytes);
        emit FibonacciVerified(n, a, b, msg.sender);
    }
}
