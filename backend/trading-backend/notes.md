- [x] Positions actor
- [x] Order endpoint
- [x] P&L calculations
- [x] Liquidations
- [ ] trading server -> redis pub-sub so the fuckers know they are liquidated.
      for this, we are already connected to a redis instance, and users are also
      connected to it for price updates through a websocket connection.
      we shall send the liquidations / stoploss / take-profit events to the user
      through this websocket, each ws connection must be connected to their user-id
      so that we can know whome to send what.
- [ ] User db table
      make changes
