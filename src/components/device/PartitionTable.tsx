import type { PartitionInfo } from "../../types/ipc";
import { formatBytes } from "../../lib/format";
import { Table, type TableColumn } from "../common/Table";

interface PartitionTableProps {
  partitions: PartitionInfo[];
  selected: ReadonlySet<string>;
  onToggle: (name: string) => void;
  onToggleAll: (names: string[]) => void;
  disabled?: boolean;
  showCheckbox?: boolean;
}

function CheckboxCell({
  checked,
  name,
  disabled,
  onToggle,
}: {
  checked: boolean;
  name: string;
  disabled: boolean;
  onToggle: (name: string) => void;
}) {
  return (
    <input
      type="checkbox"
      className="checkbox"
      checked={checked}
      disabled={disabled}
      onChange={() => onToggle(name)}
      aria-label={`Pilih partisi ${name}`}
    />
  );
}

export function PartitionTable({
  partitions,
  selected,
  onToggle,
  onToggleAll,
  disabled = false,
  showCheckbox = true,
}: PartitionTableProps) {
  const allSelected = partitions.length > 0 && partitions.every((p) => selected.has(p.name));

  const columns: TableColumn<PartitionInfo>[] = [
    ...(showCheckbox
      ? [
          {
            key: "checkbox",
            header: (
              <input
                type="checkbox"
                className="checkbox"
                checked={allSelected}
                disabled={disabled}
                onChange={() => onToggleAll(partitions.map((p) => p.name))}
                aria-label="Pilih semua partisi"
              />
            ),
            render: (row: PartitionInfo) => (
              <CheckboxCell checked={selected.has(row.name)} name={row.name} disabled={disabled} onToggle={onToggle} />
            ),
            width: "40px",
          },
        ]
      : []),
    { key: "name", header: "Partisi", width: "40%" },
    {
      key: "size",
      header: "Ukuran",
      render: (row: PartitionInfo) => <span className="mono">{formatBytes(row.size)}</span>,
    },
    {
      key: "type",
      header: "Tipe",
      render: (row: PartitionInfo) =>
        row.isUserPartition === true ? (
          <span className="tag tag--user">Data pribadi</span>
        ) : (
          <span className="tag tag--system">Sistem</span>
        ),
    },
  ];

  return (
    <Table
      columns={columns}
      rows={partitions}
      rowKey={(p) => p.name}
      emptyText="Belum ada partisi. Sambungkan device dan tekan Pindai."
    />
  );
}
