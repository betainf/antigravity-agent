import React from 'react';
import { Shield } from 'lucide-react';

interface ToolbarTitleProps {
  className?: string;
}

const ToolbarTitle: React.FC<ToolbarTitleProps> = ({ className = '' }) => {
  return (
    <h1 className={`toolbar-title text-2xl font-bold m-0 bg-gradient-to-r from-antigravity-blue to-purple-600 bg-clip-text text-transparent flex items-center gap-2 ${className}`}>
      Antigravity Agent
    </h1>
  );
};

export default ToolbarTitle;