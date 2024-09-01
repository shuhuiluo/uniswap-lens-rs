import {
  Address,
  Hex,
  PublicClient,
  RpcTransactionRequest,
  decodeFunctionResult,
  encodeFunctionData,
  toHex,
} from "viem";
import { EphemeralStorageLens__factory } from "../../typechain";

type StateOverrides = {
  [address: Address]: {
    balance?: Hex;
    nonce?: Hex;
    code?: Hex;
    stateDiff?: {
      [slot: Hex]: Hex;
    };
  };
};

/**
 * Call a contract with the given state overrides.
 * @param tx The transaction request.
 * @param overrides The state overrides.
 * @param publicClient A JSON RPC provider that supports `eth_call` with state overrides.
 * @param blockNumber Optional block number to use for the call.
 */
export async function staticCallWithOverrides(
  tx: RpcTransactionRequest,
  overrides: StateOverrides,
  publicClient: PublicClient,
  blockNumber?: bigint,
): Promise<Hex> {
  return (await publicClient.request({
    method: "eth_call",
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    params: [tx, blockNumber ? toHex(blockNumber) : "latest", overrides],
  })) as Hex;
}

/**
 * Batch `eth_getStorageAt` RPC calls in a single `eth_call` by overriding the target contract's
 * deployed bytecode with `EphemeralStorageLens`
 * @param address The contract address to fetch storage from.
 * @param slots The storage slots to fetch.
 * @param publicClient A JSON RPC provider that supports `eth_call` with state overrides.
 * @param blockNumber Optional block number to query.
 */
export async function getStorageAt(
  address: Address,
  slots: Hex[],
  publicClient: PublicClient,
  blockNumber?: bigint,
): Promise<readonly Hex[]> {
  const overrides = {
    [address]: {
      code: "0x60806040526004361061001e5760003560e01c8063dbd035ff14610023575b600080fd5b61003661003136600461007c565b61004c565b60405161004391906100f3565b60405180910390f35b60606020600052816020528160051b6040016040845b803554825260209182019101828210610062575050806000f35b6000806020838503121561008f57600080fd5b823567ffffffffffffffff8111156100a657600080fd5b8301601f810185136100b757600080fd5b803567ffffffffffffffff8111156100ce57600080fd5b8560208260051b84010111156100e357600080fd5b6020919091019590945092505050565b602080825282518282018190526000918401906040840190835b8181101561012b57835183526020938401939092019160010161010d565b50909594505050505056fea164736f6c634300081a000a" as Hex,
    },
  };
  const data = await staticCallWithOverrides(
    {
      to: address,
      from: "0x0000000000000000000000000000000000000000",
      data: encodeFunctionData({
        abi: EphemeralStorageLens__factory.abi,
        functionName: "extsload",
        args: [slots],
      }),
    },
    overrides,
    publicClient,
    blockNumber,
  );
  return decodeFunctionResult({
    abi: EphemeralStorageLens__factory.abi,
    functionName: "extsload",
    data,
  });
}
