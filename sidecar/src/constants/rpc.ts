const rpc = {
  cosmos: {
    broadcastTx: {
      description: "Broadcast cosmos transaction.",
      params: [
        {
          name: "tx_bytes",
          type: "Bytes",
        },
      ],
      type: "H256",
    },
    simulate: {
      description: "Simulate cosmos transaction.",
      params: [
        {
          name: "tx_bytes",
          type: "Bytes",
        },
      ],
      type: "SimulateResponse",
    },
  },
};

export default rpc;
