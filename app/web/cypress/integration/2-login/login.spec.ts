import { TestCtx } from "../../support/commands";

describe("Login", () => {
  beforeEach(() => {
    cy.visit("/");
    cy.signup();
  });

  it("lets the user log in", () => {
    cy.visit("authenticate/login");
    cy.get("@nw").then((testCtx: any) => {
      cy.getBySel("workspaceName").type(testCtx.workspaceName);
      cy.getBySel("userEmail").type(testCtx.userEmail);
      cy.getBySel("userPassword").type("snakes");
      cy.getBySel("login")
        .click()
        .should(() => {
          expect(localStorage.getItem("si-sdf-token")).to.not.be.null;
        });
      cy.url().should("be.match", /\/$/);
    });
  });
});
