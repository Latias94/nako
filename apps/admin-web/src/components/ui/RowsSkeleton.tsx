export function RowsSkeleton({
  label = "Loading rows",
  rows = 3,
}: {
  label?: string;
  rows?: number;
}) {
  return (
    <div aria-label={label} className="uiRowsSkeleton" role="status">
      {Array.from({ length: rows }, (_, index) => (
        <span key={index} />
      ))}
    </div>
  );
}
