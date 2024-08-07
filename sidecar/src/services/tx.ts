import { ApiPromise } from "@pinot/api";
import { ResultTx, ResultTxSearch } from "../types/index.js";
import { ApiService } from "./service.js";
import { Database } from "lmdb";
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
    const rawTx = `0x${Buffer.from(txBytes, 'base64').toString('hex')}`;

    let txHash = (await this.chainApi.rpc['cosm']['broadcastTx'](rawTx)).toString();
    if (txHash.startsWith('0x')) {
      txHash = txHash.substring(2);
    }
    console.debug(`txHash: ${txHash.toLowerCase()}`);

    await this.db.put(`tx::origin::${txHash.toLowerCase()}`, txBytes);
    return {
      txResponse: {
        height: Long.ZERO,
        txhash: txHash.toUpperCase(),
        codespace: '',
        code: 0,
        data: '',
        rawLog: '',
        logs: [],
        info: '',
        gasWanted: Long.ZERO,
        gasUsed: Long.ZERO,
        tx: {
          typeUrl: '',
          value: new Uint8Array(),
        },
        timestamp: '',
        events: [],
      },
    };
  }

  public searchTx(hash: string): ResultTxSearch {
    if (hash.startsWith('0x')) {
      hash = hash.slice(2);
    }

    console.debug(`txHash: ${hash.toLowerCase()}`);

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
    if (txRaw.startsWith('0x')) {
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
        data: '',
        log: '',
        info: '',
        gas_wanted: '0',
        gas_used: gasUsed.toString(),
        events: [],
        codespace: '',
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
          (`${section}::${method}` === 'system::ExtrinsicSuccess' ||
            `${section}::${method}` === 'system::ExtrinsicFailed')
        );
      })
      .map(({ event: { data, section, method } }) => {
        if (`${section}::${method}` === 'system::ExtrinsicSuccess') {
          const events = JSON.parse(data);
          const { refTime } = events[0].weight;

          return { code: 0, gasUsed: refTime };
        } else {
          console.debug(JSON.parse(data));

          return { code: 0, gasUsed: 0 };
        }
      });
    return result[0];
  }

  convert(str: string, from: BufferEncoding, to: BufferEncoding) {
    if (from === 'hex') {
      str = str.startsWith('0x') ? str.slice(2) : str;
    }
    return Buffer.from(str, from).toString(to);
  }

  public async simulate(txBytes: string): Promise<SimulateResponse> {
    const txRaw = `0x${this.convert(txBytes, 'base64', 'hex')}`;
    console.debug(`raw transaction: ${txRaw}`);

    const { gas_info, events } = (await this.chainApi.rpc['cosm']['simulate'](txRaw)).toJSON();

    const cosmosEvents = events.map(({ type, attributes }) => {
      const eventType = this.convert(type, 'hex', 'utf8');

      const eventAttributes = attributes.map(({ key, value }) => {
        const eventKey = this.convert(key, 'hex', 'utf8');
        const eventValue = this.convert(value, 'hex', 'utf8');

        return {
          key: eventKey,
          value: eventValue,
        }
      });

      return {
        type: eventType,
        attributes: eventAttributes,
      }
    });

    console.debug(`gasInfo: ${JSON.stringify(gas_info)}`);
    console.debug(`events: ${JSON.stringify(cosmosEvents)}`);

    return {
      gasInfo: {
        gasWanted: Long.fromNumber(gas_info.gas_wanted),
        gasUsed: Long.fromNumber(gas_info.gas_used),
      },
      result: {
        data: new Uint8Array(),
        log: '',
        events: cosmosEvents,
        msgResponses: [],
      },
    };
  }
}
