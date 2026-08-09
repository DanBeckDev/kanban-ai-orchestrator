import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "./accordion";

describe("AccordionContent", () => {
  it("disables its expand and collapse animations when reduced motion is preferred", () => {
    render(
      <Accordion defaultValue="details" type="single">
        <AccordionItem value="details">
          <AccordionTrigger>Details</AccordionTrigger>
          <AccordionContent>Task context</AccordionContent>
        </AccordionItem>
      </Accordion>,
    );

    expect(screen.getByText("Task context").parentElement).toHaveClass(
      "motion-reduce:data-open:animate-none",
      "motion-reduce:data-closed:animate-none",
    );
  });
});
