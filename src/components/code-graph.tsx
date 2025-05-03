"use client";

import { useRef, useEffect, useState } from "react";
import ForceGraph2D from "react-force-graph-2d";
import {
  Loader2,
  ZoomIn,
  ZoomOut,
  Maximize2,
  Minimize2,
  Network,
} from "lucide-react";
import { Button } from "./ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";

export interface GraphNode {
  id: string;
  name: string;
  type: "class" | "method" | "function" | "variable";
  group?: string;
  value?: number;
}

export interface GraphLink {
  source: string;
  target: string;
  type: "calls" | "imports" | "extends" | "implements" | "contains";
}

export interface CodeGraphData {
  nodes: GraphNode[];
  links: GraphLink[];
}

interface CodeGraphProps {
  codeGraph: CodeGraphData;
  isLoading: boolean;
}

export function CodeGraph({ codeGraph, isLoading }: CodeGraphProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const graphRef = useRef<any>(null);
  const [dimensions, setDimensions] = useState({ width: 800, height: 600 });
  const [filterType, setFilterType] = useState<string>("all");
  const [isFullscreen, setIsFullscreen] = useState(false);

  // Update dimensions on resize
  useEffect(() => {
    const updateDimensions = () => {
      if (containerRef.current) {
        setDimensions({
          width: containerRef.current.clientWidth,
          height: containerRef.current.clientHeight,
        });
      }
    };

    updateDimensions();
    window.addEventListener("resize", updateDimensions);

    return () => {
      window.removeEventListener("resize", updateDimensions);
    };
  }, []);

  // Filter nodes based on selected type
  const filteredData = {
    nodes:
      filterType === "all"
        ? codeGraph.nodes
        : codeGraph.nodes.filter((node) => node.type === filterType),
    links: codeGraph.links,
  };

  const handleZoomIn = () => {
    if (graphRef.current) {
      const currentZoom = graphRef.current.zoom();
      graphRef.current.zoom(currentZoom * 1.2, 400);
    }
  };

  const handleZoomOut = () => {
    if (graphRef.current) {
      const currentZoom = graphRef.current.zoom();
      graphRef.current.zoom(currentZoom * 0.8, 400);
    }
  };

  const toggleFullscreen = () => {
    if (!document.fullscreenElement) {
      containerRef.current?.requestFullscreen();
      setIsFullscreen(true);
    } else {
      document.exitFullscreen();
      setIsFullscreen(false);
    }
  };

  // Handle fullscreen change events
  useEffect(() => {
    const handleFullscreenChange = () => {
      setIsFullscreen(!!document.fullscreenElement);
    };

    document.addEventListener("fullscreenchange", handleFullscreenChange);

    return () => {
      document.removeEventListener("fullscreenchange", handleFullscreenChange);
    };
  }, []);

  const nodeColor = (node: GraphNode) => {
    switch (node.type) {
      case "class":
        return "#1A66FF"; // zed-500
      case "method":
        return "#4D8EFF"; // zed-400
      case "function":
        return "#A1F515"; // pear-400
      case "variable":
        return "#FFD97F"; // cream-500
      default:
        return "#B3D5FF"; // zed-200
    }
  };

  return (
    <div className="flex flex-col h-full" ref={containerRef}>
      <div className="p-4 border-b border-zed-100 dark:border-zed-800 flex flex-wrap gap-2 items-center justify-between">
        <div className="flex items-center gap-2">
          <Select value={filterType} onValueChange={setFilterType}>
            <SelectTrigger className="w-[180px]">
              <SelectValue placeholder="Filter by type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Types</SelectItem>
              <SelectItem value="class">Classes</SelectItem>
              <SelectItem value="method">Methods</SelectItem>
              <SelectItem value="function">Functions</SelectItem>
              <SelectItem value="variable">Variables</SelectItem>
            </SelectContent>
          </Select>

          <div className="text-sm text-zed-600 dark:text-zed-400">
            {filteredData.nodes.length} nodes
          </div>
        </div>

        <div className="flex items-center gap-2">
          <Button variant="outline" size="icon" onClick={handleZoomIn}>
            <ZoomIn className="h-4 w-4" />
          </Button>
          <Button variant="outline" size="icon" onClick={handleZoomOut}>
            <ZoomOut className="h-4 w-4" />
          </Button>
          <Button variant="outline" size="icon" onClick={toggleFullscreen}>
            {isFullscreen ? (
              <Minimize2 className="h-4 w-4" />
            ) : (
              <Maximize2 className="h-4 w-4" />
            )}
          </Button>
        </div>
      </div>

      <div className="flex-1 relative">
        {isLoading ? (
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="flex flex-col items-center gap-4">
              <Loader2 className="h-8 w-8 text-zed-500 animate-spin" />
              <p className="text-zed-600 dark:text-zed-400">
                Loading code graph...
              </p>
            </div>
          </div>
        ) : filteredData.nodes.length > 0 ? (
          <ForceGraph2D
            ref={graphRef}
            graphData={filteredData}
            width={dimensions.width}
            height={dimensions.height}
            nodeLabel="name"
            nodeColor={nodeColor}
            nodeRelSize={6}
            linkDirectionalArrowLength={4}
            linkDirectionalArrowRelPos={1}
            linkCurvature={0.25}
            cooldownTicks={100}
            onEngineStop={() => console.log("Graph layout stabilized")}
          />
        ) : (
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="flex flex-col items-center gap-4 text-center">
              <Network className="h-16 w-16 text-zed-300" />
              <div>
                <h3 className="text-xl font-medium mb-2 text-zed-600 dark:text-zed-300">
                  No Graph Data
                </h3>
                <p className="text-zed-500 dark:text-zed-400 max-w-md">
                  Select a directory to visualize its code structure
                </p>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
