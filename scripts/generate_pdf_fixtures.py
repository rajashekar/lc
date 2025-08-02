#!/usr/bin/env python3
"""
Script to generate PDF test fixtures for automated testing.
Requires: pip install reportlab PyPDF2 Pillow
"""

import os
import sys
from pathlib import Path

try:
    from reportlab.pdfgen import canvas
    from reportlab.lib.pagesizes import letter, A4
    from reportlab.lib import colors
    from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer
    from reportlab.lib.styles import getSampleStyleSheet
    import PyPDF2
    from PIL import Image, ImageDraw
except ImportError as e:
    print(f"Missing required dependencies: {e}")
    print("Please install with: pip install reportlab PyPDF2 Pillow")
    sys.exit(1)

def create_simple_text_pdf():
    """Create a simple PDF with plain text."""
    filename = "tests/pdf_fixtures/simple_text.pdf"
    c = canvas.Canvas(filename, pagesize=letter)
    
    # Title
    c.setFont("Helvetica-Bold", 16)
    c.drawString(72, 720, "Simple Text PDF")
    
    # Body text
    c.setFont("Helvetica", 12)
    text_lines = [
        "This is a simple PDF document for testing text extraction.",
        "It contains basic text that should be easily extractable.",
        "",
        "Key test points:",
        "- Plain ASCII text",
        "- Multiple lines",
        "- Standard fonts",
        "- No encryption",
        "",
        "Expected output: All text should be extracted correctly."
    ]
    
    y_position = 680
    for line in text_lines:
        c.drawString(72, y_position, line)
        y_position -= 20
    
    c.save()
    print(f"Created: {filename}")

def create_multi_page_pdf():
    """Create a multi-page PDF document."""
    filename = "tests/pdf_fixtures/multi_page.pdf"
    c = canvas.Canvas(filename, pagesize=letter)
    
    # Page 1
    c.setFont("Helvetica-Bold", 16)
    c.drawString(72, 720, "Multi-Page PDF - Page 1")
    
    c.setFont("Helvetica", 12)
    c.drawString(72, 680, "This is the first page of a multi-page document.")
    c.drawString(72, 660, "It contains text that spans multiple pages.")
    c.drawString(72, 640, "Page breaks should be handled correctly.")
    
    c.showPage()  # Start new page
    
    # Page 2
    c.setFont("Helvetica-Bold", 16)
    c.drawString(72, 720, "Multi-Page PDF - Page 2")
    
    c.setFont("Helvetica", 12)
    c.drawString(72, 680, "This is the second page.")
    c.drawString(72, 660, "Text extraction should preserve page structure.")
    c.drawString(72, 640, "Form feed characters may be used as page separators.")
    
    c.showPage()  # Start new page
    
    # Page 3
    c.setFont("Helvetica-Bold", 16)
    c.drawString(72, 720, "Multi-Page PDF - Page 3")
    
    c.setFont("Helvetica", 12)
    c.drawString(72, 680, "This is the final page.")
    c.drawString(72, 660, "End of multi-page test document.")
    
    c.save()
    print(f"Created: {filename}")

def create_encrypted_pdf():
    """Create an encrypted PDF that requires a password."""
    # First create a simple PDF
    temp_filename = "tests/pdf_fixtures/temp_unencrypted.pdf"
    c = canvas.Canvas(temp_filename, pagesize=letter)
    
    c.setFont("Helvetica-Bold", 16)
    c.drawString(72, 720, "Encrypted PDF Document")
    
    c.setFont("Helvetica", 12)
    c.drawString(72, 680, "This document is password protected.")
    c.drawString(72, 660, "Password: testpassword")
    c.drawString(72, 640, "Text extraction should fail without the password.")
    
    c.save()
    
    # Now encrypt it
    encrypted_filename = "tests/pdf_fixtures/encrypted.pdf"
    
    with open(temp_filename, 'rb') as input_file:
        pdf_reader = PyPDF2.PdfReader(input_file)
        pdf_writer = PyPDF2.PdfWriter()
        
        # Add all pages
        for page in pdf_reader.pages:
            pdf_writer.add_page(page)
        
        # Encrypt with password
        pdf_writer.encrypt("testpassword")
        
        with open(encrypted_filename, 'wb') as output_file:
            pdf_writer.write(output_file)
    
    # Clean up temp file
    os.remove(temp_filename)
    print(f"Created: {encrypted_filename} (password: testpassword)")

def create_image_only_pdf():
    """Create a PDF that contains only images (no extractable text)."""
    filename = "tests/pdf_fixtures/image_only.pdf"
    
    # Create a simple image with text
    img = Image.new('RGB', (600, 400), color='white')
    draw = ImageDraw.Draw(img)
    
    # Draw some text as image (not selectable text)
    try:
        # Try to use a better font if available
        from PIL import ImageFont
        font = ImageFont.load_default()
    except:
        font = None
    
    draw.text((50, 50), "This is text rendered as an image", fill='black', font=font)
    draw.text((50, 100), "It cannot be extracted as text", fill='black', font=font)
    draw.text((50, 150), "OCR would be needed to read this", fill='black', font=font)
    draw.text((50, 200), "Expected: [image page] or similar", fill='black', font=font)
    
    # Save as temporary image
    temp_img_path = "tests/pdf_fixtures/temp_image.png"
    img.save(temp_img_path)
    
    # Create PDF with the image
    c = canvas.Canvas(filename, pagesize=letter)
    c.drawImage(temp_img_path, 72, 400, width=400, height=300)
    c.save()
    
    # Clean up temp image
    os.remove(temp_img_path)
    print(f"Created: {filename}")

def create_complex_pdf():
    """Create a PDF with mixed content for comprehensive testing."""
    filename = "tests/pdf_fixtures/complex.pdf"
    c = canvas.Canvas(filename, pagesize=letter)
    
    # Page 1: Text content
    c.setFont("Helvetica-Bold", 16)
    c.drawString(72, 720, "Complex PDF Document")
    
    c.setFont("Helvetica", 12)
    c.drawString(72, 680, "This PDF contains various content types:")
    c.drawString(72, 660, "• Regular text")
    c.drawString(72, 640, "• Special characters: áéíóú ñ ç ü")
    c.drawString(72, 620, "• Unicode: 你好 こんにちは русский")
    c.drawString(72, 600, "• Numbers: 12345.67")
    c.drawString(72, 580, "• Symbols: @#$%^&*()_+-={}[]|\\:;\"'<>?,./")
    
    c.showPage()
    
    # Page 2: Different fonts and sizes
    c.setFont("Times-Roman", 14)
    c.drawString(72, 720, "Different fonts and formatting")
    
    c.setFont("Courier", 10)
    c.drawString(72, 680, "Monospace font text")
    c.drawString(72, 660, "Code-like content: if (x == 1) { return true; }")
    
    c.setFont("Helvetica-Oblique", 12)
    c.drawString(72, 620, "Italic text for emphasis")
    
    c.save()
    print(f"Created: {filename}")

def create_empty_pdf():
    """Create an empty PDF with no content."""
    filename = "tests/pdf_fixtures/empty.pdf"
    c = canvas.Canvas(filename, pagesize=letter)
    c.save()  # Save without adding any content
    print(f"Created: {filename}")

def main():
    """Generate all PDF test fixtures."""
    # Create fixtures directory
    fixtures_dir = Path("tests/pdf_fixtures")
    fixtures_dir.mkdir(parents=True, exist_ok=True)
    
    print("Generating PDF test fixtures...")
    
    try:
        create_simple_text_pdf()
        create_multi_page_pdf()
        create_encrypted_pdf()
        create_image_only_pdf()
        create_complex_pdf()
        create_empty_pdf()
        
        print("\nAll PDF fixtures created successfully!")
        print("\nGenerated files:")
        for pdf_file in sorted(fixtures_dir.glob("*.pdf")):
            size = pdf_file.stat().st_size
            print(f"  {pdf_file.name} ({size} bytes)")
            
    except Exception as e:
        print(f"Error generating fixtures: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
