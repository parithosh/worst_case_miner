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

        // Set all mined storage slots to 1
        assembly {
            sstore(0xc23303a7ec42f89ca5bf674b83a9141500ac18f0, 1)
            sstore(0xc5f82341cec8f50dcc8cad69423974ca8ada0f20, 1)
            sstore(0xed37894a02b4d0a25ea01b6e631a3915613f1bb5, 1)
            sstore(0xbcfbe3c33c48e394b37f4c11e79d7d77bcae2e24, 1)
            sstore(0xa1e0359fb56f177056221a1207081f04aea9cd79, 1)
            sstore(0xbeebb5327db7f26d86063a8f5e530faed72b3abc, 1)
            sstore(0x291133eb5b3f634f88e28ace53113de696bb07e9, 1)
            sstore(0x491fabf0e5edc6f78440cb9d0c7d33c162fab471, 1)
            sstore(0x8c78d0234248e4e5daf92d722ad7826b26830ee6, 1)
            sstore(0x98f699eb65d72e429ecab836545372ac4e3479f6, 1)
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
            sstore(0x98f699eb65d72e429ecab836545372ac4e3479f6, value)
        }
    }

    // Optional: getter to verify the deepest slot value
    function getDeepest() external view returns (uint256 value) {
        assembly {
            value := sload(0x98f699eb65d72e429ecab836545372ac4e3479f6)
        }
    }
}