interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  label?: string;
  description?: string;
}

export function Toggle({ checked, onChange, disabled = false, label, description }: ToggleProps) {
  return (
    <label className={`toggle${disabled ? " toggle--disabled" : ""}`}>
      <input
        type="checkbox"
        className="toggle__input"
        checked={checked}
        disabled={disabled}
        onChange={(e) => onChange(e.currentTarget.checked)}
      />
      <span className="toggle__track" aria-hidden="true">
        <span className="toggle__thumb" />
      </span>
      {(label != null || description != null) && (
        <span className="toggle__text">
          {label != null && <span className="toggle__label">{label}</span>}
          {description != null && <span className="toggle__desc">{description}</span>}
        </span>
      )}
    </label>
  );
}
