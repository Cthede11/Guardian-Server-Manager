import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useConsoleStream } from "@/store/live-new";

interface ConsoleStreamProps {
  serverId: string;
}

export default function ConsoleStreamNew({ serverId }: ConsoleStreamProps) {
  const parentRef = useRef<HTMLDivElement>(null);
  const lines = useConsoleStream(serverId);
  
  const rowVirtualizer = useVirtualizer({
    count: lines.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 18,
    overscan: 30,
  });

  const items = rowVirtualizer.getVirtualItems();
  const paddingTop = items.length ? items[0].start : 0;
  const paddingBottom = items.length ? rowVirtualizer.getTotalSize() - items[items.length - 1].end : 0;

  return (
    <div ref={parentRef} className="h-full overflow-auto font-mono text-sm">
      <div style={{ height: rowVirtualizer.getTotalSize(), position: "relative" }}>
        <div style={{ transform: `translateY(${paddingTop}px)` }}>
          {items.map((virtualItem) => {
            const line = lines[virtualItem.index];
            return (
              <div key={virtualItem.key} className="leading-5">
                <span className="text-zinc-400">
                  {new Date(line.ts).toLocaleTimeString()}{" "}
                </span>
                <span 
                  className={
                    line.level === "ERROR" 
                      ? "text-red-400" 
                      : line.level === "WARN" 
                      ? "text-amber-400" 
                      : line.level === "DEBUG"
                      ? "text-zinc-500"
                      : ""
                  }
                >
                  [{line.level}]
                </span>{" "}
                <span>{line.msg}</span>
              </div>
            );
          })}
        </div>
      </div>
      <div style={{ height: paddingBottom }} />
    </div>
  );
}
