// Risky.sol — demo fixture whose fallback executes SSTORE + DELEGATECALL,
// so RiskInspector's storage_slots_written and delegatecalls_detected are
// genuinely non-zero (real opcode hits, not faked output).
//
// The delegatee address is a constant: no storage setup needed at runtime.

pragma solidity ^0.8.20;

contract Delegatee {
    uint256 public poked;

    // Fallback so a delegatecall with ANY calldata performs an SSTORE here
    // (executed in the caller's storage context).
    fallback() external payable {
        poked += 1;
    }
}

contract Risky {
    // Fixed delegatee so DELEGATECALL target needs no runtime setup.
    address constant DELEGATEE = 0xc0Ffee0000000000000000000000000000000002;

    uint256 public counter;

    fallback() external payable {
        counter += 1; // SSTORE (0x55) in Risky's storage
        assembly {
            // DELEGATECALL (0xF4) into Delegatee — its fallback SSTOREs,
            // so the inspector sees a real delegatecall + storage write.
            calldatacopy(0, 0, calldatasize())
            let ok := delegatecall(gas(), DELEGATEE, 0, calldatasize(), 0, 0)
            pop(ok)
        }
    }

    // Explicit DELEGATECALL path: this contract also proxies any calldata it
    // receives through delegatecall so the inspector sees opcode 0xF4.
    function proxyDelegate(bytes memory data) external {
        assembly {
            let success := delegatecall(gas(), DELEGATEE, add(data, 32), mload(data), 0, 0)
            pop(success)
        }
    }
}
