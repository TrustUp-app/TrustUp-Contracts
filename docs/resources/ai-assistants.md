# AI Assistants & MCP Servers for Stellar/Soroban

AI-powered development assistants and Model Context Protocol (MCP) servers for enhanced Soroban development.

## What is MCP?

**Model Context Protocol (MCP)** is an open protocol that enables seamless integration between LLM applications (like Claude) and external data sources and tools.

Think of MCP as USB-C for AI - a standardized way for AI assistants to connect to different tools and services.

## Available MCP Servers for Stellar

### 1. Stellar MCP (syronlabs/stellar-mcp)

Complete MCP server for Stellar blockchain interaction.

**Features**:
- Interact with Horizon API
- Invoke Soroban smart contract functions
- Manage accounts and operations
- Query blockchain state
- Execute transactions

**Installation**:
```bash
npm install @mseep/stellar-mcp
```

**GitHub**: https://github.com/syronlabs/stellar-mcp

**Configuration for Claude Desktop**:

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "stellar": {
      "command": "npx",
      "args": ["@mseep/stellar-mcp"],
      "env": {}
    }
  }
}
```

### 2. OpenZeppelin Stellar Contracts MCP

Generates secure smart contracts based on OpenZeppelin templates.

**Features**:
- Generate audited contract code
- Based on OpenZeppelin patterns
- Returns code in Markdown format
- No disk writes (safe)

**Access**:
- Web Wizard: https://wizard.openzeppelin.com/stellar
- Docker: https://hub.docker.com/mcp/server/openzeppelin-stellar

### 3. Soroban-to-MCP Converter

Tool that converts any Soroban smart contract into an MCP server.

**Use Case**: Transform your deployed contracts into AI-callable interfaces

**Features**:
- Reads Soroban contract specifications
- Generates compliant MCP server
- Supports Stellar Asset Contracts (SAC)
- Works with custom WASM contracts

**Link**: https://dorahacks.io/buidl/25271

### 4. Stellar Blockchain MCP Server (UBOS)

Comprehensive blockchain operations server.

**Features**:
- Standard interface for Stellar operations
- Account management
- Transaction processing
- Contract interaction

**Link**: https://ubos.tech/mcp/stellar-blockchain-mcp-server/

## Setup Guide

### Prerequisites

1. **Claude Desktop** installed
2. **Node.js** (for npm packages)
3. **MCP-compatible client**

### Step 1: Install Stellar MCP

```bash
# Install the package
npm install -g @mseep/stellar-mcp

# Verify installation
npx @mseep/stellar-mcp --version
```

### Step 2: Configure Claude Desktop

1. Open config file:
```bash
code ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

2. Add Stellar MCP configuration:
```json
{
  "mcpServers": {
    "stellar": {
      "command": "npx",
      "args": ["@mseep/stellar-mcp"],
      "env": {
        "STELLAR_NETWORK": "testnet"
      }
    }
  }
}
```

3. Restart Claude Desktop

### Step 3: Verify Connection

In Claude, ask:
```
Can you show me my Stellar MCP capabilities?
```

Claude should be able to list available Stellar operations.

## TrustUp Configuration

For TrustUp development, we provide a pre-configured file:

**File**: `/claude_desktop_config.json` (project root)

Copy this to your Claude Desktop config:
```bash
cp claude_desktop_config.json ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

Or merge with existing config if you have other MCP servers.

## Usage Examples

### With Stellar MCP

**Query account balance**:
```
Check the balance of account GXXXXXXX...
```

**Invoke contract function**:
```
Call the get_score function on contract CXXXXXXX with address GXXXXXXX
```

**Deploy contract**:
```
Deploy the reputation-contract.wasm to testnet
```

### With OpenZeppelin MCP

**Generate contract**:
```
Generate an SEP-41 token contract with the following parameters:
- Name: TrustToken
- Symbol: TRUST
- Decimals: 7
- Admin: <address>
```

### Convert Your Contract to MCP

```bash
# Using the Soroban-to-MCP tool
npx stellar-mcp-cli --contract <contract-id> --network testnet
```

This generates an MCP server that allows Claude to interact with your contract using natural language.

## What You Can Do

With MCP servers configured, you can ask Claude to:

1. **Develop Contracts**:
   - "Generate a secure token contract using OpenZeppelin"
   - "Add access control to this contract"

2. **Test Contracts**:
   - "Deploy my contract to testnet"
   - "Call the increase_score function with these parameters"

3. **Query Blockchain**:
   - "What's the current score for this address?"
   - "Show me recent transactions for this contract"

4. **Debug Issues**:
   - "Why did this transaction fail?"
   - "Check if this account exists on testnet"

5. **Analyze Code**:
   - "Review this contract for security issues"
   - "Suggest improvements using OpenZeppelin patterns"

## Best Practices

### 1. Use Testnet First
Always configure MCP for testnet during development:
```json
{
  "env": {
    "STELLAR_NETWORK": "testnet"
  }
}
```

### 2. Secure Your Keys
Never put private keys in MCP config. Use environment variables or key management:
```json
{
  "env": {
    "STELLAR_SECRET_KEY": "${STELLAR_SECRET_KEY}"
  }
}
```

### 3. Version Lock
Pin MCP package versions in package.json:
```json
{
  "dependencies": {
    "@mseep/stellar-mcp": "^1.0.0"
  }
}
```

### 4. Test MCP Integration
Verify MCP commands work before relying on them:
```bash
# Test stellar-mcp directly
npx @mseep/stellar-mcp test-connection
```

## Troubleshooting

### MCP Not Working

1. Check config file syntax (valid JSON)
2. Restart Claude Desktop
3. Verify npm package installed: `npm list -g @mseep/stellar-mcp`
4. Check Claude Desktop logs: `~/Library/Logs/Claude/`

### Connection Issues

1. Verify network access (testnet/mainnet reachable)
2. Check environment variables set correctly
3. Try running MCP command directly: `npx @mseep/stellar-mcp`

### Contract Invocation Fails

1. Verify contract deployed on correct network
2. Check function signature matches
3. Ensure account has sufficient balance
4. Review transaction errors in Stellar Expert

## Resources

### Documentation
- [MCP Specification](https://modelcontextprotocol.io/)
- [Claude MCP Setup Guide](https://code.claude.com/docs/en/mcp)
- [Stellar MCP GitHub](https://github.com/syronlabs/stellar-mcp)

### Catalogs
- [MCP Servers Catalog](https://mcpservers.org/)
- [Awesome MCP Servers](https://github.com/punkpeye/awesome-mcp-servers)

### Community
- [Stellar Discord](https://discord.gg/stellar) - #soroban channel
- [MCP GitHub Discussions](https://github.com/modelcontextprotocol/modelcontextprotocol/discussions)

## Next Steps

1. ✅ Install Stellar MCP
2. ✅ Configure Claude Desktop
3. ✅ Test connection
4. Try generating contracts with OpenZeppelin wizard
5. Convert TrustUp contracts to MCP servers
6. Build natural language interfaces for your dApp

## Future Possibilities

- **Custom MCP for TrustUp**: Convert reputation/creditline contracts to MCP
- **Natural Language Testing**: Test contracts with conversational commands
- **Automated Audits**: Use Claude + MCP for security analysis
- **Documentation Generation**: Auto-generate docs from contract specs
