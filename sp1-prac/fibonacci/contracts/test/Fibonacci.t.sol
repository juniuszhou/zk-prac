// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {Fibonacci} from "../src/Fibonacci.sol";
import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";

contract MockSP1Verifier is ISP1Verifier {
    function verifyProof(bytes32, bytes calldata, bytes calldata) external pure {}
}

contract MockSP1RevertVerifier is ISP1Verifier {
    function verifyProof(bytes32, bytes calldata, bytes calldata) external pure {
        revert("bad proof");
    }
}

/// @dev Matches `SP1FibonacciProofFixture` JSON from `script/src/bin/evm.rs`.
struct SP1ProofFixtureJson {
    uint32 a;
    uint32 b;
    uint32 n;
    bytes proof;
    bytes publicValues;
    bytes32 vkey;
}

abstract contract FibonacciFixtureTest is Test {
    using stdJson for string;

    Fibonacci public fibonacci;
    address public verifier;

    function fixturePath() internal pure virtual returns (string memory);

    function loadFixture() public view returns (SP1ProofFixtureJson memory) {
        string memory path = string.concat(vm.projectRoot(), fixturePath());
        string memory json = vm.readFile(path);
        return SP1ProofFixtureJson({
            a: uint32(json.readUint(".a")),
            b: uint32(json.readUint(".b")),
            n: uint32(json.readUint(".n")),
            proof: json.readBytes(".proof"),
            publicValues: json.readBytes(".publicValues"),
            vkey: json.readBytes32(".vkey")
        });
    }

    function setUp() public virtual {
        SP1ProofFixtureJson memory fixture = loadFixture();
        verifier = address(new MockSP1Verifier());
        fibonacci = new Fibonacci(verifier, fixture.vkey);
    }

    function test_ValidFibonacciProof() public view {
        SP1ProofFixtureJson memory fixture = loadFixture();

        (uint32 n, uint32 a, uint32 b) =
            fibonacci.verifyFibonacciProof(fixture.publicValues, fixture.proof);
        assertEq(n, fixture.n);
        assertEq(a, fixture.a);
        assertEq(b, fixture.b);
    }
}

contract FibonacciGroth16Test is FibonacciFixtureTest {
    function fixturePath() internal pure override returns (string memory) {
        return "/src/fixtures/groth16-fixture.json";
    }
}

contract FibonacciPlonkTest is FibonacciFixtureTest {
    function fixturePath() internal pure override returns (string memory) {
        return "/src/fixtures/plonk-fixture.json";
    }
}

contract FibonacciRevertTest is Test {
    using stdJson for string;

    Fibonacci public fibonacci;

    function loadFixture() internal view returns (SP1ProofFixtureJson memory) {
        string memory path = string.concat(vm.projectRoot(), "/src/fixtures/groth16-fixture.json");
        string memory json = vm.readFile(path);
        return SP1ProofFixtureJson({
            a: uint32(json.readUint(".a")),
            b: uint32(json.readUint(".b")),
            n: uint32(json.readUint(".n")),
            proof: json.readBytes(".proof"),
            publicValues: json.readBytes(".publicValues"),
            vkey: json.readBytes32(".vkey")
        });
    }

    function setUp() public {
        SP1ProofFixtureJson memory fixture = loadFixture();
        fibonacci = new Fibonacci(address(new MockSP1RevertVerifier()), fixture.vkey);
    }

    function testRevert_WhenVerifierRejects() public {
        SP1ProofFixtureJson memory fixture = loadFixture();
        vm.expectRevert("bad proof");
        fibonacci.verifyFibonacciProof(fixture.publicValues, fixture.proof);
    }
}
