import { test, expect } from "@playwright/test";

test("restructuredtext", async ({ page }) => {
  await page.goto("/en/docs/demo/main/restructuredtext");

  await expect(page.getByRole("article")).toMatchAriaSnapshot(`
    - table:
      - rowgroup:
        - row "Header row, column 1 (header rows optional) Header 2 Header 3 Header 4":
          - cell "Header row, column 1 (header rows optional)":
            - paragraph: Header row, column 1 (header rows optional)
          - cell "Header 2":
            - paragraph: Header 2
          - cell "Header 3":
            - paragraph: Header 3
          - cell "Header 4":
            - paragraph: Header 4
      - rowgroup:
        - row "body row 1, column 1 column 2 column 3 column 4":
          - cell "body row 1, column 1":
            - paragraph: body row 1, column 1
          - cell "column 2":
            - paragraph: column 2
          - cell "column 3":
            - paragraph: column 3
          - cell "column 4":
            - paragraph: column 4
        - row "body row 2 … …":
          - cell "body row 2":
            - paragraph: body row 2
          - cell "…":
            - paragraph: …
          - cell "…":
            - paragraph: …
          - cell
    `);

  await expect(page.getByRole("article")).toMatchAriaSnapshot(`
    - term:
      - paragraph: Date
    - definition:
      - paragraph: /\\d+-\\d+-\\d+/
    - term:
      - paragraph: Version
    - definition:
      - paragraph: "1"
    - term:
      - paragraph: Authors
    - definition:
      - list:
        - listitem: Me
        - listitem: Myself
        - listitem: I
    - term:
      - paragraph: Indentation
    - definition:
      - paragraph: Since the field marker may be quite long, the second and subsequent lines of the field body do not have to line up with the first line, but they must be indented relative to the field name marker, and they must line up with each other.
    - term:
      - paragraph: Parameter i
    - definition:
      - paragraph: integer
    `);

  await expect(page.getByRole("article")).toMatchAriaSnapshot(`
    - term:
      - strong: term 1
    - definition:
      - paragraph: Definition 1.
    - term:
      - strong: term 2
    - definition:
      - paragraph: Definition 2, paragraph 1.
      - paragraph: Definition 2, paragraph 2.
    - term:
      - strong: term 3
      - emphasis: ": classifier"
    - definition:
      - paragraph: Definition 3.
    - term:
      - strong: term 4
      - emphasis: ": classifier one"
      - emphasis: ": classifier two"
    - definition:
      - paragraph: Definition 4.
    `);

  await expect(page.getByRole("article")).toMatchAriaSnapshot(`
    - term: "-a"
    - definition:
      - paragraph: Output all.
    - term: "-c arg"
    - definition:
      - paragraph: Output just arg.
    - term: "--long"
    - definition:
      - paragraph: Output all day long.
    - term: /V
    - definition:
      - paragraph: A VMS/DOS-style option.
    - term: "-p"
    - definition:
      - paragraph: This option has two paragraphs in the description. This is the first.
      - paragraph: This is the second. Blank lines may be omitted between options (as above) or left in (as here and below).
    - term: "--very-long-option"
    - definition:
      - paragraph: A VMS-style option. Note the adjustment for the required two spaces.
    - term: "--an-even-longer-option"
    - definition:
      - paragraph: The description can also start on the next line.
    - term:
      - paragraph: "-2, --two"
    - definition:
      - paragraph: This option has two variants.
    - term:
      - paragraph: "-f FILE, --file=FILE"
    - definition:
      - paragraph: These two options are synonyms; both have arguments.
    - term: "-f <[path]file>"
    - definition:
      - paragraph: Option argument placeholders must start with a letter or be wrapped in angle brackets.
    - term: "-d <src dest>"
    - definition:
      - paragraph: Angle brackets are also required if an option expects more than one argument.
    `);
});
