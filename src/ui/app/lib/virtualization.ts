import type { Dispatch, ReactNode, SetStateAction } from "react";
import { useMemo, useState } from "react";

export interface RenderVirtualItem {
  height: number;
  item: ReactNode;
}

export const useVirtualSection = ({
  heading: headingFn,
  items,
  initialOpen,
}: {
  heading: (
    open: boolean,
    setOpen: Dispatch<SetStateAction<boolean>>,
  ) => RenderVirtualItem;
  items: RenderVirtualItem[];
  initialOpen: boolean;
}) => {
  const [open, setOpen] = useState(initialOpen);
  const heading = useMemo(
    () => headingFn(open, setOpen),
    [headingFn, open, setOpen],
  );
  const output = useMemo(
    () => [heading, ...(open ? items : [])],
    [heading, items, open],
  );
  return { output, open, setOpen };
};

export const useConcatVirtualItems = (
  items1: RenderVirtualItem[],
  items2: RenderVirtualItem[],
) => {
  return useMemo(() => [...items1, ...items2], [items1, items2]);
};

export const usePrefixVirtualItems = (
  prefix: RenderVirtualItem,
  items: RenderVirtualItem[],
) => {
  return useMemo(() => [prefix, ...items], [prefix, items]);
};

export const useSuffixVirtualItems = (
  items: RenderVirtualItem[],
  suffix: RenderVirtualItem,
) => {
  return useMemo(() => [...items, suffix], [items, suffix]);
};
