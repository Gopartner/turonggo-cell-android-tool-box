import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import type { DeviceStatus, PartitionInfo } from "../types/ipc";
import * as api from "../lib/api";
import { listenDeviceStatus } from "../lib/events";

interface DeviceContextValue {
  status: DeviceStatus;
  partitions: PartitionInfo[];
  isLoading: boolean;
  scan: () => Promise<void>;
}

const DeviceContext = createContext<DeviceContextValue | null>(null);

const INITIAL_STATUS: DeviceStatus = { mode: "none" };

export function DeviceProvider({ children }: { children: ReactNode }) {
  const [status, setStatus] = useState<DeviceStatus>(INITIAL_STATUS);
  const [partitions, setPartitions] = useState<PartitionInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const scan = useCallback(async () => {
    setIsLoading(true);
    try {
      const [device, parts] = await Promise.all([api.getDeviceStatus(), api.partitionList()]);
      setStatus(device);
      setPartitions(parts);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void scan();
    const un = listenDeviceStatus((next) => {
      setStatus(next);
      if (next.mode === "unisoc_download") void api.partitionList().then(setPartitions).catch(() => {});
    });
    return () => {
      void un.then((fn) => fn());
    };
  }, [scan]);

  const value = useMemo(
    () => ({ status, partitions, isLoading, scan }),
    [status, partitions, isLoading, scan],
  );

  return <DeviceContext.Provider value={value}>{children}</DeviceContext.Provider>;
}

export function useDevice(): DeviceContextValue {
  const ctx = useContext(DeviceContext);
  if (!ctx) throw new Error("useDevice harus dipakai di dalam DeviceProvider");
  return ctx;
}
