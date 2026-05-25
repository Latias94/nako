import type { ReactNode } from "react";

export function RoutePage({
  actions,
  children,
  description,
  kicker,
  status,
  title,
  titleId,
}: {
  actions?: ReactNode;
  children: ReactNode;
  description?: ReactNode;
  kicker: string;
  status?: ReactNode;
  title: string;
  titleId: string;
}) {
  return (
    <section className="routePage" aria-labelledby={titleId}>
      <div className="routeHeader">
        <div>
          <p className="routeKicker">{kicker}</p>
          <div className="routeTitleLine">
            <h1 id={titleId}>{title}</h1>
            {status}
          </div>
          {description ? <p>{description}</p> : null}
        </div>
        {actions}
      </div>
      {children}
    </section>
  );
}

export function RouteNotice({ children }: { children: ReactNode }) {
  return (
    <div className="routeNotice" role="status">
      {children}
    </div>
  );
}

export function EmptyRouteState({ children }: { children: ReactNode }) {
  return <div className="emptyRouteState">{children}</div>;
}
