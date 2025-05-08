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
import type { GraphNode, CodeGraphData } from "@/lib/types";

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

  // Update dimensions on resize and component mount
  useEffect(() => {
    const updateDimensions = () => {
      if (containerRef.current) {
        const { clientWidth, clientHeight } = containerRef.current;
        // Ensure we have meaningful dimensions
        const width = Math.max(clientWidth, 300);
        const height = Math.max(clientHeight, 300);

        setDimensions({ width, height });
      }
    };

    // Initial update
    updateDimensions();

    // Set a timeout to ensure container has fully rendered
    const timer = setTimeout(updateDimensions, 100);

    // Update on resize
    window.addEventListener("resize", updateDimensions);

    return () => {
      window.removeEventListener("resize", updateDimensions);
      clearTimeout(timer);
    };
  }, []);

  // Filter nodes based on selected type
  const filteredData = {
    nodes:
      filterType === "all"
        ? codeGraph.nodes
        : codeGraph.nodes.filter((node) => node.type === filterType),
    links: codeGraph.links.filter((link) => {
      if (filterType === "all") return true;
      // Only include links where both source and target pass the filter
      const sourceNode = codeGraph.nodes.find(
        (node) => node.id === link.source,
      );
      const targetNode = codeGraph.nodes.find(
        (node) => node.id === link.target,
      );
      return (
        sourceNode &&
        targetNode &&
        (filterType === "all" ||
          sourceNode.type === filterType ||
          targetNode.type === filterType)
      );
    }),
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
      // Update dimensions when entering/exiting fullscreen
      if (containerRef.current) {
        const { clientWidth, clientHeight } = containerRef.current;
        setDimensions({
          width: Math.max(clientWidth, 300),
          height: Math.max(clientHeight, 300),
        });
      }
    };

    document.addEventListener("fullscreenchange", handleFullscreenChange);

    return () => {
      document.removeEventListener("fullscreenchange", handleFullscreenChange);
    };
  }, []);

  // Neo4j-like node color palette
  const nodeColor = (node: GraphNode) => {
    switch (node.type) {
      case "class":
        return "#4C8EDA"; // Neo4j blue
      case "method":
        return "#57C7E3"; // Neo4j light blue
      case "function":
        return "#F16667"; // Neo4j red
      case "variable":
        return "#D9C8AE"; // Neo4j beige
      case "file":
        return "#8DCC93"; // Neo4j green
      case "directory":
        return "#ECB5C9"; // Neo4j pink
      case "import":
        return "#FFC454"; // Neo4j yellow
      default:
        return "#C990C0"; // Neo4j purple
    }
  };

  // Calculate node size based on importance (Neo4j-like)
  const nodeSize = (node: any) => {
    if (node.isProjectRoot) return 8; // Make the project root node larger
    switch (node.type) {
      case "directory":
        return 6;
      case "file":
        return 5;
      case "class":
        return 4;
      case "function":
      case "method":
        return 3;
      default:
        return 2;
    }
  };

  // Fix initial layout and scaling
  useEffect(() => {
    if (graphRef.current && filteredData.nodes.length > 0) {
      // Initial zoom out to see everything
      graphRef.current.zoom(0.7, 0);

      // Center the graph
      graphRef.current.centerAt(0, 0);

      // Automatically fit the graph when data changes
      setTimeout(() => {
        graphRef.current.zoomToFit(500, 150); // duration, padding
      }, 500);
    }
  }, [filteredData.nodes.length]);

  return (
    <div className="flex flex-col h-full w-full" ref={containerRef}>
      <div className="p-4 border-b border-zed-100 dark:border-zed-800 flex flex-wrap gap-2 items-center justify-between">
        <div className="flex items-center gap-2">
          <Select value={filterType} onValueChange={setFilterType}>
            <SelectTrigger className="w-[180px]">
              <SelectValue placeholder="Filter by type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Types</SelectItem>
              <SelectItem value="directory">Directories</SelectItem>
              <SelectItem value="file">Files</SelectItem>
              <SelectItem value="class">Classes</SelectItem>
              <SelectItem value="method">Methods</SelectItem>
              <SelectItem value="function">Functions</SelectItem>
              <SelectItem value="variable">Variables</SelectItem>
              <SelectItem value="import">Imports</SelectItem>
              <SelectItem value="callsite">Call Sites</SelectItem>
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

      <div className="flex-1 relative w-full" style={{ minHeight: "500px" }}>
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
            backgroundColor="#fafafa" // Light background like Neo4j Browser
            nodeLabel={(node) => `${node.name} (${node.type})`}
            nodeColor={nodeColor}
            nodeRelSize={3} // Smaller base node size (Neo4j style)
            nodeVal={(node) => nodeSize(node)} // Use node size function for varying sizes
            // Neo4j-like link styling
            linkWidth={1.5} // Thicker links
            linkColor={() => "#A5ABB6"} // Neo4j default link color
            linkDirectionalArrowLength={6} // Larger arrows
            linkDirectionalArrowRelPos={1}
            linkCurvature={0.2} // Slight curve
            linkDirectionalParticles={2} // Add flowing particles on links (Neo4j-like)
            linkDirectionalParticleWidth={1.5} // Particle width
            linkDirectionalParticleSpeed={0.01} // Slow speed for particles
            linkLabel={(link) => link.type}
            // Text styling
            nodeCanvasObjectMode={() => "after"}
            nodeCanvasObject={(node, ctx, globalScale) => {
              const label = node.name;
              const fontSize = 12 / globalScale;
              ctx.font = `${fontSize}px Sans-Serif`;
              ctx.textAlign = "center";
              ctx.textBaseline = "middle";
              ctx.fillStyle = "rgba(0,0,0,0.8)";

              // Only render text if we're zoomed in enough
              if (globalScale > 0.4) {
                // Background for text (Neo4j style)
                const textWidth = ctx.measureText(label).width;
                const bckgDimensions = [textWidth + 8, fontSize + 4].map(
                  (n) => n,
                );

                ctx.fillStyle = "rgba(255, 255, 255, 0.8)";
                ctx.fillRect(
                  node.x - bckgDimensions[0] / 2,
                  node.y + 8,
                  bckgDimensions[0],
                  bckgDimensions[1],
                );

                // Text
                ctx.fillStyle = "#333333";
                ctx.fillText(label, node.x, node.y + 8 + bckgDimensions[1] / 2);
              }
            }}
            cooldownTicks={100}
            onEngineStop={() => {
              // Make sure to fit graph after physics simulation stops
              if (graphRef.current) {
                graphRef.current.zoomToFit(400, 150);
              }
            }}
            onNodeClick={(node) => {
              // Center view on clicked node
              if (graphRef.current) {
                graphRef.current.centerAt(node.x, node.y, 1000);
                graphRef.current.zoom(1.5, 1000);
              }
              console.log("Clicked node:", node);
            }}
            // Tweak force settings for better layout
            d3AlphaDecay={0.02}
            d3VelocityDecay={0.15}
            warmupTicks={50}
            cooldownTime={2000}
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
