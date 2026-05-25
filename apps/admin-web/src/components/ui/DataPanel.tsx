import type { ReactNode } from "react";

export function DataPanel({
  children,
  description,
  headerAccessory,
  title,
}: {
  children: ReactNode;
  description?: ReactNode;
  headerAccessory?: ReactNode;
  title: string;
}) {
  return (
    <div className="dataPanel">
      <div className="dataPanelHeader">
        <div>
          <h2>{title}</h2>
          {description ? <p>{description}</p> : null}
        </div>
        {headerAccessory}
      </div>
      {children}
    </div>
  );
}
