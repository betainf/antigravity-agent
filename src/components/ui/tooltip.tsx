import * as Tooltip from '@radix-ui/react-tooltip';
import * as React from 'react';

interface TooltipProviderProps {
  children: React.ReactNode;
  delayDuration?: number;
}

const TooltipProvider: React.FC<TooltipProviderProps> = ({ children, delayDuration = 300 }) => (
  <Tooltip.Provider delayDuration={delayDuration}>
    {children}
  </Tooltip.Provider>
);

interface TooltipContentProps {
  className?: string;
  side?: 'top' | 'right' | 'bottom' | 'left';
  sideOffset?: number;
  align?: 'start' | 'center' | 'end';
  children: React.ReactNode;
  collisionPadding?: number | Partial<Record<'top' | 'right' | 'bottom' | 'left', number>>;
}

const TooltipContent = React.forwardRef<
  React.ElementRef<typeof Tooltip.Content>,
  TooltipContentProps
>(
  (
    {
      className,
      sideOffset = 4,
      side = 'top',
      align = 'center',
      collisionPadding,
      children,
      ...props
    },
    ref
  ) => (
    <Tooltip.Content
      ref={ref}
      sideOffset={sideOffset}
      side={side}
      align={align}
      collisionPadding={collisionPadding}
      className={`
        z-50 overflow-hidden rounded-md bg-gray-900 px-3 py-1.5 text-sm text-gray-50
        animate-in fade-in-0 zoom-in-95
        data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95
        data-[side=bottom]:slide-in-from-top-2
        data-[side=left]:slide-in-from-right-2
        data-[side=right]:slide-in-from-left-2
        data-[side=top]:slide-in-from-bottom-2
        ${className || ''}
      `.trim()}
      {...props}
    >
      {children}
    </Tooltip.Content>
  )
);

TooltipContent.displayName = Tooltip.Content.displayName;

interface TooltipArrowProps {
  width?: number;
  height?: number;
  className?: string;
}

const TooltipArrow = React.forwardRef<
  React.ElementRef<typeof Tooltip.Arrow>,
  TooltipArrowProps
>(({ width = 10, height = 5, className, ...props }, ref) => (
  <Tooltip.Arrow
    ref={ref}
    width={width}
    height={height}
    className={`fill-gray-900 ${className || ''}`}
    {...props}
  />
));

TooltipArrow.displayName = Tooltip.Arrow.displayName;

// 标准的 tooltip 组件
interface StandardTooltipProps {
  children: React.ReactNode;
  content: React.ReactNode;
  side?: 'top' | 'right' | 'bottom' | 'left';
  delayDuration?: number;
  className?: string;
}

export const StandardTooltip: React.FC<StandardTooltipProps> = ({
  children,
  content,
  side = 'top',
  delayDuration = 300,
  className
}) => (
  <Tooltip.Root delayDuration={delayDuration}>
    <Tooltip.Trigger asChild>
      {children}
    </Tooltip.Trigger>
    <Tooltip.Portal>
      <TooltipContent side={side} className={className}>
        {content}
        <TooltipArrow />
      </TooltipContent>
    </Tooltip.Portal>
  </Tooltip.Root>
);

export {
  TooltipProvider,
  TooltipContent,
  TooltipArrow,
};

export default {
  Provider: TooltipProvider,
  Content: TooltipContent,
  Arrow: TooltipArrow,
  StandardTooltip,
};