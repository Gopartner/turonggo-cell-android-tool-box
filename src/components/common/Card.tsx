import type { HTMLAttributes, ReactNode } from "react";

interface CardProps extends Omit<HTMLAttributes<HTMLDivElement>, "title"> {
  title?: ReactNode;
  subtitle?: ReactNode;
  actions?: ReactNode;
  children: ReactNode;
}

export function Card({ title, subtitle, actions, children, className = "", ...rest }: CardProps) {
  const cls = ["card", className].filter(Boolean).join(" ");
  return (
    <section className={cls} {...rest}>
      {(title != null || actions != null) && (
        <header className="card__header">
          <div>
            {title != null && <h3 className="card__title">{title}</h3>}
            {subtitle != null && <p className="card__subtitle">{subtitle}</p>}
          </div>
          {actions != null && <div className="card__actions">{actions}</div>}
        </header>
      )}
      <div className="card__body">{children}</div>
    </section>
  );
}
