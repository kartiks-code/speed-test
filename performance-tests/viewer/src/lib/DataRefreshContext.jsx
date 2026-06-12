import React, { createContext, useContext, useEffect, useState } from "react";
import { subscribeEvents } from "./api.js";
import { clearDataCache } from "./data.js";

const DataRefreshContext = createContext({ refreshToken: 0, lastRunId: null });

export function DataRefreshProvider({ children }) {
  const [refreshToken, setRefreshToken] = useState(0);
  const [lastRunId, setLastRunId] = useState(null);

  useEffect(() => {
    const unsub = subscribeEvents((event) => {
      if (event.type === "data_updated") {
        clearDataCache(event.runId ?? null);
        setLastRunId(event.runId ?? null);
        setRefreshToken((t) => t + 1);
      }
    });
    return unsub;
  }, []);

  return (
    <DataRefreshContext.Provider value={{ refreshToken, lastRunId }}>
      {children}
    </DataRefreshContext.Provider>
  );
}

export function useDataRefresh() {
  return useContext(DataRefreshContext);
}
