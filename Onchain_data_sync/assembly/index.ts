import { PersistentVector, context } from "near-sdk-as";

export const transactions = new PersistentVector<TransactionRecord>("TRANSACTIONS");

@nearBindgen
export class TransactionRecord {
  txn_type: string;
  purpose: string;
  amount: u64;
  user: string;
  reference: string;
  balance_before: u64;
  balance_after: u64;
  status: string;
  description: string;
  createdAt: string;
  updatedAt: string;
}

export function setTransaction(
  txn_type: string,
  purpose: string,
  amount: u64,
  user: string,
  reference: string,
  balance_before: u64,
  balance_after: u64,
  status: string,
  description: string,
  createdAt: string,
  updatedAt: string
): void {
  const transaction: TransactionRecord = {
    txn_type,
    purpose,
    amount,
    user,
    reference,
    balance_before,
    balance_after,
    status,
    description,
    createdAt,
    updatedAt,
  };
  transactions.push(transaction);
}

export function getTransactions(): TransactionRecord[] {
  const allTransactions: TransactionRecord[] = [];
  for (let i = 0; i < transactions.length; i++) {
    allTransactions.push(transactions[i]);
  }
  return allTransactions;
}

export function getTransactionsByUser(user: string): TransactionRecord[] {
  const userTransactions: TransactionRecord[] = [];
  for (let i = 0; i < transactions.length; i++) {
    if (transactions[i].user === user) {
      userTransactions.push(transactions[i]);
    }
  }
  return userTransactions;
}

export function getTransactionById(reference: string): TransactionRecord | null {
  for (let i = 0; i < transactions.length; i++) {
    if (transactions[i].reference === reference) {
      return transactions[i];
    }
  }
  return null;
}

// export function updateTransactionStatus(reference: string, status: string): boolean {
//   for (let i = 0; i < transactions.length; i++) {
//     if (transactions[i].reference === reference) {
//       transactions[i].status = status;
//       transactions[i].updatedAt = context.blockTimestamp.toString();
//       return true;
//     }
//   }
//   return false;
// }

// near call save.adashifin.testnet setTransaction '{"txn_type": "payment", "purpose": "repayment", "amount": "20000", "user": "Bala", "reference": "value2", "balance_before": "1000", "balance_after": "3000", "status": "Done", "description": "value2", "createdAt": "value2", "updatedAt": "value2"}' --accountId adashifin.testnet 


// Deploy instruction
//near deploy  --accountId=adhtest.testnet --wasmFile=build/release/staker.wasm --initFunction init --initArgs '{ " txn_type": "payment", "purpose": "repayment", "amount": "20000", "user": "Bala", "reference": "value2", "balance_before": "value2", "balance_after": "value2", "status": "value2", "description": "value2", "createdAt": "value2", "updatedAt": "value2"   }' --accountId adhtest.testnet
