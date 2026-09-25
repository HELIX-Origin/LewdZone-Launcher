import "@testing-library/jest-dom/vitest";
import { beforeEach } from "vitest";
import { resetClientCache } from "../stores/clientCache";

beforeEach(() => {
  resetClientCache();
});