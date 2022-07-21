module.exports = {
  title: "Diem",
  tagline: "A New Global Payment System",
  baseUrl: "/",
  organizationName: "diem",
  projectName: "diem",
  themeConfig: {
    logo: {
      alt: "Diem Logo",
      src: "/img/shared/logo.svg",
      to: "https://www.diem.com/",
    },
    footer: {
      links: [
        {
          items: [
            {
              label: "Vision",
              to: "https://www.diem.com/vision/",
            },
            {
              label: "About Us",
              to: "https://www.diem.com/association/",
            },
            {
              label: "Developers",
              to: "https://developers.diem.com/docs/welcome-to-diem/",
            },
            {
              label: "Learn",
              to: "https://www.diem.com/learn-faqs/",
            },
          ],
        },
        {
          items: [
            {
              label: "Media",
              to: "https://www.diem.com/media-press-news/",
            },
            {
              label: "White Paper",
              to: "https://www.diem.com/white-paper/",
            },
            {
              label: "Careers",
              to: "https://www.diem.com/careers/",
            },
          ],
        },
        {
          items: [
            {
              type: "secondary",
              label: "Privacy",
              to: "https://www.diem.com/privacy/",
            },
            {
              type: "secondary",
              label: "Cookies",
              to: "https://www.diem.com/privacy/#cookies_policy",
            },
            {
              type: "secondary",
              label: "Terms of Use",
              to: "https://www.diem.com/privacy/#terms_of_use",
            },
            {
              type: "secondary",
              label: "Code of Conduct",
              to: "https://developers.diem.com/docs/policies/code-of-conduct",
            },
          ],
        },
      ],
      copyright: `©${new Date().getFullYear()} Diem Association`,
    },
  },
  customFields: {
    socialLinks: {
      facebook: "https://www.facebook.com/diemdevelopers/",
      linkedIn: "https://www.linkedin.com/company/diemassociation/",
      twitter: "https://twitter.com/diemdevelopers/",
      instagram: "https://www.instagram.com/diemassociation/",
      github: "https://github.com/diem",
    },
    navbar: {
      primaryLinks: [
        {
          label: "Vision",
          to: "https://www.diem.com/vision/",
        },
        {
          label: "About Us",
          to: "https://www.diem.com/association/",
        },
        {
          id: "developers",
          label: "Developers",
          to: "https://developers.diem.com/docs/welcome-to-diem/",
        },
        {
          label: "Learn",
          to: "https://www.diem.com/learn-faqs/",
        },
        {
          label: "Media",
          to: "https://www.diem.com/media-press-news/",
        },
      ],
      cornerLink: {
        label: "White Paper",
        to: "https://www.diem.com/white-paper/",
        image: {
          alt: "Diem Whitepaper",
        },
      },
      secondaryLinks: [
        {
          id: "developers",
          label: "Diem Documentation",
          to: "https://developers.diem.com/docs/welcome-to-diem/",
        },
        {
          label: "Governance",
          to: "/docs/governance",
        },
        {
          isExternal: true,
          label: "GitHub",
          to: "https://github.com/diem/",
        },
      ],
    },
  },
};
