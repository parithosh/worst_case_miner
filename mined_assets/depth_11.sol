// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract WorstCaseERC20 {
    // ERC20 State
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;
    uint256 public totalSupply;

    // Token metadata - returning constants to save gas
    string public constant name = "WorstCase";
    string public constant symbol = "WORST";
    uint8 public constant decimals = 18;

    constructor() {
        // Mint total supply to deployer
        totalSupply = 1_000_000_000 * 10 ** 18; // 1 billion tokens
        balanceOf[msg.sender] = totalSupply;

        // Set all mined storage addresses to 1
        assembly {
            sstore(0x40ba25ab43e6082776eb25b94d5600905625ff87, 1)
            sstore(0x8590b076d6f58c618430185abeb5563c549c506d, 1)
            sstore(0x8ca09fcffcd4817ba633147cef3c568ec360fc3f, 1)
            sstore(0x624514fa26b457541342e4e98ddad4a519baafa5, 1)
            sstore(0x1c5cf64000d283973150fb3594b66c89f27e6100, 1)
            sstore(0x7e41b61c7a862525787957614bbbbda79534fbeb, 1)
            sstore(0xce530dddcec0b1054252929a09d190d2c8ec3fec, 1)
            sstore(0x22919d70aaa457ff74356323fcd2881c0fac755e, 1)
            sstore(0xca7ccb0a8e94fa962df8b06682408f58d75003d9, 1)
            sstore(0x0a7f33ef2f147cf8cf58cd26dda13574f6659e61, 1)
            sstore(0x6095ab050cdbe20e107cb3ef0d688df3c69bb47e, 1)
        }
    }

    // Minimal ERC20 implementation
    function transfer(address to, uint256 amount) public returns (bool) {
        require(balanceOf[msg.sender] >= amount, "Insufficient balance");
        balanceOf[msg.sender] -= amount;
        balanceOf[to] += amount;
        return true;
    }

    function approve(address spender, uint256 amount) public returns (bool) {
        allowance[msg.sender][spender] = amount;
        return true;
    }

    function transferFrom(
        address from,
        address to,
        uint256 amount
    ) public returns (bool) {
        require(balanceOf[from] >= amount, "Insufficient balance");
        require(
            allowance[from][msg.sender] >= amount,
            "Insufficient allowance"
        );

        balanceOf[from] -= amount;
        balanceOf[to] += amount;
        allowance[from][msg.sender] -= amount;

        return true;
    }

    // Attack method - writes to the deepest storage slot
    function attack(uint256 value) external {
        assembly {
            sstore(0x6095ab050cdbe20e107cb3ef0d688df3c69bb47e, value)
        }
    }

    // Optional: getter to verify the deepest slot value
    function getDeepest() external view returns (uint256 value) {
        assembly {
            value := sload(0x6095ab050cdbe20e107cb3ef0d688df3c69bb47e)
        }
    }
}
