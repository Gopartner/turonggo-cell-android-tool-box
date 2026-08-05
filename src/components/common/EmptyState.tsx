import type { ReactNode } from "react";
import { Icon, type IconName } from "./Icon";

interface EmptyStateProps {
  icon?: IconName;
  title: string;
  description?: string;
  action?: ReactNode;
}

export function EmptyState({ icon = "info", title, description, action }: EmptyStateProps) {
  return (
    <div className="empty-state">
      <div className="empty-state__icon">
        <Icon name={icon} size={26} />
      </div>
      <h4 className="empty-state__title">{title}</h4>
      {description != null && <p className="empty-state__desc">{description}</p>}
      {action != null && <div className="empty-state__action">{action}</div>}
    </div>
  );
}
