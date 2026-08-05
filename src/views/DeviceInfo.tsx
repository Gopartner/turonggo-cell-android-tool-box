import { useDevice } from "../hooks/useDevice";
import { PageHeader } from "../components/layout/PageHeader";
import { Button } from "../components/common/Button";
import { Icon } from "../components/common/Icon";
import { DeviceInfo } from "../components/device/DeviceInfo";

export function DeviceInfoView() {
  const { isLoading, scan } = useDevice();

  return (
    <div className="view">
      <PageHeader
        title="Informasi Perangkat"
        subtitle="Deteksi detail perangkat, protokol, dan diagnostik driver (Advanced Mode)"
        actions={
          <Button
            variant="secondary"
            size="sm"
            icon={<Icon name="scan" size={15} />}
            onClick={() => void scan()}
            disabled={isLoading}
          >
            Pindai Ulang
          </Button>
        }
      />
      <DeviceInfo />
    </div>
  );
}
