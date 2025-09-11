import nodemailer from "nodemailer";

const transporter = nodemailer.createTransport({
  service: "gmail",
  auth: {
    user: "atharvparlikar@gmail.com",
    pass: "dmobgcsuuzftshpw",
  },
});

export async function sendMail(to: string, token: string) {
  try {
    await transporter.sendMail({
      from: '"Atharv Parlikar" <atharvparlikar@gmail.com>',
      to,
      subject: "Magic link",
      html: `
<a href="${process.env.server_url}/signin/post?token=${token}/" target="_blank">click here</a>
`
    });
  } catch {
    return false;
  }

  return true;
}

// grxz ocfs trfd oggt
