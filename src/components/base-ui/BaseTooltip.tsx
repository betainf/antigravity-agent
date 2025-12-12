import React from 'react';
import * as Tooltip from '@radix-ui/react-tooltip';
import { cn } from '@/lib/utils.ts';

export interface BaseTooltipProps {
  children: React.ReactNode;
  content: React.ReactNode;
  side?: 'top' | 'right' | 'bottom' | 'left';
  delayDuration?: number;
  className?: string;
}

const BaseTooltipContent = React.forwardRef<
  React.ElementRef<typeof Tooltip.Content>,
  React.ComponentPropsWithoutRef<typeof Tooltip.Content>
>(
  (
    {
      className,
      sideOffset = 4,
      children,
      ...props
    },
    ref
  ) => (
    <Tooltip.Content
      ref={ref}
      sideOffset={sideOffset}
      className={cn(
        'z-50 overflow-hidden rounded-md bg-gray-900 px-3 py-1.5 text-sm text-gray-50',
        'animate-in fade-in-0 zoom-in-95',
        'data-[state=closed]:animate-out',
        'data-[state=closed]:fade-out-0',
        'data-[state=closed]:zoom-out-95',
        'data-[side=bottom]:slide-in-from-top-2',
        'data-[side=left]:slide-in-from-right-2',
        'data-[side=right]:slide-in-from-left-2',
        'data-[side=top]:slide-in-from-bottom-2',
        className
      )}
      {...props}
    >
      {children}
      <Tooltip.Arrow className="fill-gray-900" />
    </Tooltip.Content>
  )
);

BaseTooltipContent.displayName = Tooltip.Content.displayName;

/**
 * BaseUI: BaseTooltip
 * 基础工具提示组件，基于 Radix UI Tooltip 封装
 */
const BaseTooltip: React.FC<BaseTooltipProps> = ({
  children,
  content,
  side = 'top',
  delayDuration = 300,
  className
}) => {
  return (
    <Tooltip.Provider delayDuration={delayDuration}>
      <Tooltip.Root>
        <Tooltip.Trigger asChild>
          {children}
        </Tooltip.Trigger>
        <Tooltip.Portal>
          <BaseTooltipContent side={side} className={className}>
            {content}
          </BaseTooltipContent>
        </Tooltip.Portal>
      </Tooltip.Root>
    </Tooltip.Provider>
  );
};

export { BaseTooltip };
