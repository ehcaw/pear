"use client";

import React from "react";
import * as AccordionPrimitive from "@radix-ui/react-accordion";
import { ScrollArea } from "./ui/scroll-area";
import { cn } from "@/lib/utils";
import { ChevronRight, type LucideIcon } from "lucide-react";
// Remove useResizeObserver as we'll rely on flexbox/parent height now
// import useResizeObserver from "use-resize-observer";

interface TreeDataItem {
  id: string;
  name: string;
  icon?: LucideIcon;
  children?: TreeDataItem[];
  // Add type potentially needed by FileExplorer logic later
  type?: "file" | "directory";
}

type TreeProps = React.HTMLAttributes<HTMLDivElement> & {
  data: TreeDataItem[] | TreeDataItem;
  initialSlelectedItemId?: string;
  onSelectChange?: (item: TreeDataItem | undefined) => void;
  expandAll?: boolean;
  folderIcon?: LucideIcon;
  itemIcon?: LucideIcon;
  // Add level prop for indentation
  level?: number;
};

const Tree = React.forwardRef<HTMLDivElement, TreeProps>(
  (
    {
      data,
      initialSlelectedItemId,
      onSelectChange,
      expandAll,
      folderIcon,
      itemIcon,
      className,
      ...props
    },
    ref,
  ) => {
    const [selectedItemId, setSelectedItemId] = React.useState<
      string | undefined
    >(initialSlelectedItemId);

    // Update selected item if initial prop changes
    React.useEffect(() => {
      setSelectedItemId(initialSlelectedItemId);
    }, [initialSlelectedItemId]);

    const handleSelectChange = React.useCallback(
      (item: TreeDataItem | undefined) => {
        setSelectedItemId(item?.id);
        if (onSelectChange) {
          onSelectChange(item);
        }
      },
      [onSelectChange],
    );

    // Memoize expanded IDs based on initial selection or expandAll flag
    const expandedItemIds = React.useMemo(() => {
      if (expandAll) {
        // Collect all directory IDs if expandAll is true
        const dirIds: string[] = [];
        function collectDirIds(items: TreeDataItem[] | TreeDataItem) {
          const nodes = Array.isArray(items) ? items : [items];
          nodes.forEach((node) => {
            if (node.children && node.children.length > 0) {
              dirIds.push(node.id);
              collectDirIds(node.children); // Recurse
            }
          });
        }
        collectDirIds(data);
        return dirIds;
      }

      if (!initialSlelectedItemId) {
        return [] as string[];
      }

      // Logic to find path to the initial selected item
      const ids: string[] = [];
      function findPath(
        currentItems: TreeDataItem[] | TreeDataItem,
        targetId: string,
        currentPath: string[],
      ): boolean {
        const itemsArray = Array.isArray(currentItems)
          ? currentItems
          : [currentItems];

        for (const item of itemsArray) {
          if (item.id === targetId) {
            ids.push(...currentPath, item.id); // Add ancestors and the item itself if it's a folder
            return true;
          }
          if (item.children) {
            if (findPath(item.children, targetId, [...currentPath, item.id])) {
              return true; // Found in children
            }
          }
        }
        return false; // Not found in this branch
      }

      findPath(data, initialSlelectedItemId, []);
      // We only want the IDs of the *ancestor folders*, not the target item itself unless it's also a folder
      // The Accordion 'defaultValue' needs the IDs of the Accordion.Item elements (folders) to open.
      return ids.filter((id) => {
        function isDir(
          items: TreeDataItem[] | TreeDataItem,
          itemId: string,
        ): boolean {
          const nodes = Array.isArray(items) ? items : [items];
          for (const node of nodes) {
            if (node.id === itemId) return !!node.children;
            if (node.children && isDir(node.children, itemId)) return true;
          }
          return false;
        }
        return isDir(data, id);
      });
    }, [data, initialSlelectedItemId, expandAll]);

    // REMOVED useResizeObserver and ScrollArea wrapper here.
    // Scrolling will be handled by the parent (FileExplorer)

    return (
      // This div will contain the root ul/li structure
      <div
        ref={ref}
        className={cn("w-full", className)} // Ensure it takes width, let parent handle height/scroll
      >
        <TreeItem
          data={data}
          selectedItemId={selectedItemId}
          handleSelectChange={handleSelectChange}
          expandedItemIds={expandedItemIds}
          FolderIcon={folderIcon}
          ItemIcon={itemIcon}
          level={0} // Start recursion at level 0
          {...props}
        />
      </div>
    );
  },
);
Tree.displayName = "Tree";

type TreeItemProps = TreeProps & {
  selectedItemId?: string;
  handleSelectChange: (item: TreeDataItem | undefined) => void;
  expandedItemIds: string[];
  FolderIcon?: LucideIcon;
  ItemIcon?: LucideIcon;
  level: number; // Added level prop
};

const TreeItem = React.forwardRef<HTMLUListElement, TreeItemProps>(
  (
    {
      className,
      data,
      selectedItemId,
      handleSelectChange,
      expandedItemIds,
      FolderIcon,
      ItemIcon,
      level, // Use level for indentation/styling
      ...props
    },
    ref,
  ) => {
    return (
      // Use ul for the list structure
      <ul ref={ref} role="tree" className={cn("space-y-1", className)}>
        {Array.isArray(data) ? (
          data.map((item) => (
            <li key={item.id}>
              {item.children && item.children.length > 0 ? ( // Check children reliably
                <AccordionPrimitive.Root
                  type="multiple"
                  // Use expandedItemIds directly - these should be the folder IDs
                  defaultValue={expandedItemIds}
                  className="w-full"
                >
                  <AccordionPrimitive.Item
                    value={item.id}
                    className="border-none"
                  >
                    <AccordionTrigger
                      className={cn(
                        "px-2 py-1.5 hover:bg-muted/80 rounded-md relative group", // Adjusted styling
                        // Highlight logic
                        selectedItemId === item.id &&
                          "bg-accent text-accent-foreground",
                      )}
                      // Apply indentation based on level
                      style={{ paddingLeft: `${level * 1.25 + 0.5}rem` }} // Indent trigger
                      onClick={(e) => {
                        e.stopPropagation(); // Prevent triggering parent selection
                        handleSelectChange(item);
                      }}
                    >
                      {/* Icon rendering */}
                      {item.icon ? (
                        <item.icon
                          className="h-4 w-4 shrink-0 mr-2 text-muted-foreground group-hover:text-foreground"
                          aria-hidden="true"
                        />
                      ) : FolderIcon ? (
                        <FolderIcon
                          className="h-4 w-4 shrink-0 mr-2 text-muted-foreground group-hover:text-foreground"
                          aria-hidden="true"
                        />
                      ) : null}
                      <span className="text-sm truncate flex-grow">
                        {item.name}
                      </span>
                      {/* Chevron is now part of AccordionTrigger */}
                    </AccordionTrigger>
                    <AccordionContent
                      // Don't add extra padding here, rely on TreeItem's padding
                      className="overflow-hidden" // Ensure content doesn't leak bounds during animation
                    >
                      {/* Recursively render children */}
                      <TreeItem
                        data={item.children} // Pass children array
                        selectedItemId={selectedItemId}
                        handleSelectChange={handleSelectChange}
                        expandedItemIds={expandedItemIds}
                        FolderIcon={FolderIcon}
                        ItemIcon={ItemIcon}
                        level={level + 1} // Increment level
                      />
                    </AccordionContent>
                  </AccordionPrimitive.Item>
                </AccordionPrimitive.Root>
              ) : (
                // Render Leaf for files or empty directories
                <Leaf
                  item={item}
                  isSelected={selectedItemId === item.id}
                  onClick={(e) => {
                    e.stopPropagation();
                    handleSelectChange(item);
                  }}
                  Icon={item.icon ?? ItemIcon} // Use specific icon or default ItemIcon
                  level={level} // Pass level for indentation
                />
              )}
            </li>
          ))
        ) : (
          // Handle case where data is a single item (though less common for root)
          <li>
            <Leaf
              item={data}
              isSelected={selectedItemId === data.id}
              onClick={(e) => {
                e.stopPropagation();
                handleSelectChange(data);
              }}
              Icon={data.icon ?? ItemIcon}
              level={level}
            />
          </li>
        )}
      </ul>
    );
  },
);
TreeItem.displayName = "TreeItem";

// Leaf Component remains largely the same, adding indentation
const Leaf = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & {
    item: TreeDataItem;
    isSelected?: boolean;
    Icon?: LucideIcon;
    level: number; // Added level
  }
>(({ className, item, isSelected, Icon, level, ...props }, ref) => {
  return (
    <div
      ref={ref}
      className={cn(
        "flex items-center py-1.5 px-2 cursor-pointer rounded-md hover:bg-muted/80 group", // Adjusted styling
        className,
        isSelected && "bg-accent text-accent-foreground",
      )}
      style={{ paddingLeft: `${level * 1.25 + 1.5}rem` }} // Indent leaves more (icon width + folder indent)
      {...props}
    >
      {/* Icon rendering */}
      {Icon && (
        <Icon
          className="h-4 w-4 shrink-0 mr-2 text-muted-foreground group-hover:text-foreground"
          aria-hidden="true"
        />
      )}
      <span className="text-sm truncate">{item.name}</span>
    </div>
  );
});
Leaf.displayName = "Leaf";

// AccordionTrigger: Ensure chevron rotates correctly and styling is applied
const AccordionTrigger = React.forwardRef<
  React.ElementRef<typeof AccordionPrimitive.Trigger>,
  React.ComponentPropsWithoutRef<typeof AccordionPrimitive.Trigger>
>(({ className, children, ...props }, ref) => (
  <AccordionPrimitive.Header className="flex">
    <AccordionPrimitive.Trigger
      ref={ref}
      className={cn(
        "flex w-full items-center text-left py-0 transition-all [&[data-state=open]>svg]:rotate-90", // Removed last:, py-2
        className,
      )}
      {...props}
    >
      {/* Chevron is now inside the main children flex container */}
      <ChevronRight className="h-4 w-4 shrink-0 transition-transform duration-200 text-muted-foreground mr-2" />
      {children}
    </AccordionPrimitive.Trigger>
  </AccordionPrimitive.Header>
));
AccordionTrigger.displayName = AccordionPrimitive.Trigger.displayName;

// AccordionContent: Ensure smooth animation
const AccordionContent = React.forwardRef<
  React.ElementRef<typeof AccordionPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof AccordionPrimitive.Content>
>(({ className, children, ...props }, ref) => (
  <AccordionPrimitive.Content
    ref={ref}
    className={cn(
      // Use Tailwind animations defined in globals.css (or similar setup)
      "overflow-hidden text-sm data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down",
      className,
    )}
    {...props}
  >
    {/* Remove extra padding div, rely on TreeItem/Leaf padding */}
    {children}
  </AccordionPrimitive.Content>
));
AccordionContent.displayName = AccordionPrimitive.Content.displayName;

export { Tree, type TreeDataItem };
