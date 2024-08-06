import { ApiPromise } from "@pinot/api";
import { ResultTx, ResultTxSearch } from "../types/index.js";
import { ApiService } from "./service.js";
import { Database } from "lmdb";
import { Tx } from "cosmjs-types/cosmos/tx/v1beta1/tx.js";
import Weights from "../constants/weights.js";
import {
  BroadcastTxResponse,
  SimulateResponse,
} from "cosmjs-types/cosmos/tx/v1beta1/service.js";
import Long from "long";
import { createHash } from "crypto";

type TransactResult = { code: number; gasUsed: number };

export class TxService implements ApiService {
  chainApi: ApiPromise;
  db: Database;

  constructor(db: Database, chainApi: ApiPromise) {
    this.chainApi = chainApi;
    this.db = db;
  }

  public async broadcastTx(txBytes: string): Promise<BroadcastTxResponse> {
    const rawTx = `0x${Buffer.from(txBytes, "base64").toString("hex")}`;
    console.debug(`raw transaction: ${rawTx} `)

    const res = await this.chainApi.rpc["cosm"]["broadcastTx"](rawTx);
    let txhash = res.toString();

    if (txhash.startsWith("0x")) {
      txhash = txhash.substring(2);
    }

    await this.db.put(`tx::origin::${txhash.toLowerCase()}`, txBytes);
    return {
      txResponse: {
        height: Long.ZERO,
        txhash: txhash.toUpperCase(),
        codespace: "",
        code: 0,
        data: "",
        rawLog: "",
        logs: [],
        info: "",
        gasWanted: Long.ZERO,
        gasUsed: Long.ZERO,
        tx: {
          typeUrl: "",
          value: new Uint8Array(),
        },
        timestamp: "",
        events: [],
      },
    };
  }

  public searchTx(hash: string): ResultTxSearch {
    const resultTx = this.db.get(`tx::result::${hash.toLowerCase()}`);
    const txs: ResultTx[] = [];
    if (resultTx) {
      txs.push(resultTx);
    }
    return {
      txs,
      total_count: txs.length,
    };
  }

  public async saveTransactResult(
    txRaw: string,
    extrinsicIndex: number,
    header: any
  ): Promise<void> {
    if (txRaw.startsWith("0x")) {
      txRaw = txRaw.substring(2);
    }
    const txHash = createHash('sha256').update(Buffer.from(txRaw, 'hex')).digest('hex');

    const rawTx = this.db.get(`tx::origin::${txHash.toLowerCase()}`);
    const { code, gasUsed } = await this.checkResult(header, extrinsicIndex);
    const txResult: ResultTx = {
      hash: `${txHash.toUpperCase()}`,
      height: header.number.toString(),
      index: extrinsicIndex,
      tx_result: {
        code,
        data: "",
        log: "",
        info: "",
        gas_wanted: 0,
        gas_used: gasUsed,
        events: [],
        codespace: "",
      },
      tx: rawTx,
    };
    await this.db.put(`tx::result::${txHash.toLowerCase()}`, txResult);
  }

  async checkResult(
    header: any,
    extrinsicIndex: number
  ): Promise<TransactResult> {
    const events = (await (
      await this.chainApi.at(header.hash)
    ).query.system.events()) as any;
    const result = events
      .filter(({ event: { section, method }, phase }) => {
        const { applyExtrinsic } = JSON.parse(phase.toString());
        return (
          applyExtrinsic === extrinsicIndex &&
          (`${section}::${method}` === "cosmos::Executed" ||
            `${section}::${method}` === "system::ExtrinsicFailed")
        );
      })
      .map(({ event: { data, section, method } }) => {
        if (`${section}::${method}` === "cosmos::Executed") {
          const result = JSON.parse(data.toString());
          const code = result[0];
          const { refTime } = result[1];
          return { code, gasUsed: refTime };
        } else {
          const _data = JSON.parse(data.toString());
          console.debug({ _data });
          // const { refTime } = JSON.parse(data.toString())[1]["weight"];
          // let code = error;
          // if (code.startsWith("0x")) {
          //   code = code.substring(2);
          // }
          // code = Buffer.from(code, "hex").readUint32LE();

          return { code: 0, gasUsed: 0 };
        }
      });
    return result[0];
  }

  public async simulate(txBytes: string): Promise<SimulateResponse> {
    const rawTx = `0x${Buffer.from(txBytes, "base64").toString("hex")}`;
    console.debug(`raw transaction: ${rawTx} `)

    const res = await this.chainApi.rpc["cosm"]["simulate"](rawTx);
    return {
      gasInfo: {
        gasWanted: Long.fromNumber(0),
        gasUsed: Long.fromNumber(0),
      },
      result: {
        data: new Uint8Array(),
        log: "",
        events: [],
        msgResponses: [],
      },
    };
  }
}
