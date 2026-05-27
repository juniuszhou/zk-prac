// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

interface IAdditionGroth16Verifier {
    function verifyProof(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[1] calldata pubSignals
    ) external view returns (bool);
}

contract AdditionProofApp {
    IAdditionGroth16Verifier public immutable verifier;

    event AdditionProofVerified(uint256 indexed publicSum, bool valid);

    constructor(address verifier_) {
        require(verifier_ != address(0), "verifier is zero address");
        verifier = IAdditionGroth16Verifier(verifier_);
    }

    function verifyAddition(
        uint256 publicSum,
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC
    ) external returns (bool valid) {
        uint256[1] memory publicSignals = [publicSum];

        valid = verifier.verifyProof(pA, pB, pC, publicSignals);
        emit AdditionProofVerified(publicSum, valid);
    }

    function verifyAdditionView(
        uint256 publicSum,
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC
    ) external view returns (bool) {
        uint256[1] memory publicSignals = [publicSum];

        return verifier.verifyProof(pA, pB, pC, publicSignals);
    }
}
