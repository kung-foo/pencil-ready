import * as React from "react";
import { Accordion as AccordionPrimitive } from "radix-ui";
import { ChevronDown } from "lucide-react";

import { cn } from "@/lib/utils";

function Accordion({
    ...props
}: React.ComponentProps<typeof AccordionPrimitive.Root>) {
    return <AccordionPrimitive.Root data-slot="accordion" {...props} />;
}

function AccordionItem({
    className,
    ...props
}: React.ComponentProps<typeof AccordionPrimitive.Item>) {
    return (
        <AccordionPrimitive.Item
            data-slot="accordion-item"
            className={cn("border-b last:border-b-0", className)}
            {...props}
        />
    );
}

function AccordionTrigger({
    className,
    children,
    ...props
}: React.ComponentProps<typeof AccordionPrimitive.Trigger>) {
    return (
        <AccordionPrimitive.Header className="flex">
            <AccordionPrimitive.Trigger
                data-slot="accordion-trigger"
                className={cn(
                    // The default AccordionPrimitive.Header is <h3>, which some
                    // projects retarget to a display font via global CSS. Force the
                    // body font here so triggers always match sibling <Label>s.
                    "flex flex-1 items-center justify-between gap-4 py-2 text-left text-sm font-medium leading-none outline-none transition-colors focus-visible:ring-[3px] focus-visible:ring-ring/50 [font-family:var(--font-sans)] [&[data-state=open]>svg]:rotate-180",
                    className,
                )}
                {...props}
            >
                {children}
                <ChevronDown className="pointer-events-none size-4 shrink-0 text-muted-foreground transition-transform duration-200" />
            </AccordionPrimitive.Trigger>
        </AccordionPrimitive.Header>
    );
}

function AccordionContent({
    className,
    children,
    ...props
}: React.ComponentProps<typeof AccordionPrimitive.Content>) {
    return (
        <AccordionPrimitive.Content
            data-slot="accordion-content"
            className="overflow-hidden text-sm"
            {...props}
        >
            <div className={cn("pt-1 pb-3", className)}>{children}</div>
        </AccordionPrimitive.Content>
    );
}

export { Accordion, AccordionItem, AccordionTrigger, AccordionContent };
